from __future__ import annotations

import json
import logging
import math
import os
import time
from datetime import datetime, timedelta, timezone
from pathlib import Path

from .config import ResearchConfig
from .evaluate import SavedMultimodalPredictor

logger = logging.getLogger("xauusd_model.worker")


def _required_env(name: str) -> str:
    value = os.getenv(name, "").strip()
    if not value:
        raise RuntimeError(f"{name} is not configured")
    return value


def _latest_closed_decision(config: ResearchConfig) -> datetime | None:
    import clickhouse_connect

    client = clickhouse_connect.get_client(url=config.clickhouse_url)
    try:
        row = client.query(
            """SELECT max(time) AS candle_time
               FROM market.ohlcv_candles_15m
               WHERE symbol = 'XAUUSD'
                 AND time + INTERVAL 15 MINUTE <= now64(3, 'UTC')"""
        ).first_row
    finally:
        client.close()
    if not row or row[0] is None:
        return None
    candle_time = row[0]
    if candle_time.tzinfo is None:
        candle_time = candle_time.replace(tzinfo=timezone.utc)
    return candle_time.astimezone(timezone.utc) + timedelta(minutes=15)


def _unprocessed_decisions(config: ResearchConfig, model_version: str) -> list[datetime]:
    import clickhouse_connect
    import psycopg

    with psycopg.connect(config.postgres_url) as connection:
        last_stored = connection.execute(
            """SELECT max(decision_at) FROM market.market_forecasts
               WHERE model_version = %s AND symbol = 'XAUUSD' AND timeframe = '15m'""",
            (model_version,),
        ).fetchone()[0]
    latest = _latest_closed_decision(config)
    if latest is None:
        return []
    if last_stored is None:
        return [latest]

    client = clickhouse_connect.get_client(url=config.clickhouse_url)
    try:
        rows = client.query(
            """SELECT DISTINCT time
               FROM market.ohlcv_candles_15m
               WHERE symbol = 'XAUUSD'
                 AND time > {last_stored:DateTime64(3, 'UTC')} - INTERVAL 15 MINUTE
                 AND time + INTERVAL 15 MINUTE <= now64(3, 'UTC')
               ORDER BY time ASC""",
            parameters={"last_stored": last_stored},
        ).result_rows
    finally:
        client.close()
    decisions = []
    for (candle_time,) in rows:
        if candle_time.tzinfo is None:
            candle_time = candle_time.replace(tzinfo=timezone.utc)
        decision_at = candle_time.astimezone(timezone.utc) + timedelta(minutes=15)
        if last_stored < decision_at <= latest:
            decisions.append(decision_at)
    return decisions


def _insert_forecast(postgres_url: str, row: dict) -> None:
    import psycopg
    from psycopg.types.json import Jsonb

    with psycopg.connect(postgres_url) as connection:
        connection.execute(
            """INSERT INTO market.market_forecasts (
                   symbol, timeframe, decision_at, horizon_end, reference_price,
                   down_probability, flat_probability, up_probability, expected_return,
                   q10_return, q50_return, q90_return, uncertainty, model_version,
                   data_version, feature_version, source_masks, source_age_seconds
               ) VALUES (
                   'XAUUSD', '15m', %(decision_at)s, %(horizon_end)s, %(reference_price)s,
                   %(down_probability)s, %(flat_probability)s, %(up_probability)s, %(expected_return)s,
                   %(q10_return)s, %(q50_return)s, %(q90_return)s, %(uncertainty)s, %(model_version)s,
                   %(data_version)s, %(feature_version)s, %(source_masks)s, %(source_age_seconds)s
               ) ON CONFLICT (model_version, symbol, decision_at) DO NOTHING""",
            {
                **row,
                "source_masks": Jsonb(row["source_masks"]),
                "source_age_seconds": Jsonb(row["source_age_seconds"]),
            },
        )


def _runtime_config(run_dir: Path) -> tuple[ResearchConfig, dict]:
    manifest = json.loads((run_dir / "run.json").read_text())
    values = dict(manifest.get("config") or {})
    values.update(
        postgres_url=_required_env("TRADING_MODEL_POSTGRES_URL"),
        clickhouse_url=_required_env("TRADING_MODEL_CLICKHOUSE_URL"),
        artifact_dir=os.getenv("TRADING_MODEL_CACHE_DIR", "/worker-cache"),
    )
    fields = ResearchConfig.__dataclass_fields__
    return ResearchConfig(**{key: value for key, value in values.items() if key in fields}), manifest


def _forecast_for(config: ResearchConfig, predictor: SavedMultimodalPredictor, manifest: dict, decision_at: datetime) -> dict:
    import pandas as pd

    from .cli import _build_panel

    as_of = decision_at.isoformat()
    context_start = (decision_at - timedelta(minutes=15 * (config.sequence_length - 1))).isoformat()
    panel = _build_panel(config, context_start, as_of)
    panel = panel[pd.to_datetime(panel["decision_at"], utc=True) == pd.Timestamp(decision_at)].reset_index(drop=True)
    if panel.empty:
        raise ValueError(f"no feature row available for closed candle at {as_of}")
    values = predictor.predict_latest(panel)
    row = panel.iloc[-1]
    price = float(row["close"])
    if not math.isfinite(price) or price <= 0:
        raise ValueError("XAUUSD reference price must be positive")
    if not all(math.isfinite(value) for value in values.values()):
        raise ValueError("model produced non-finite forecast values")
    if not values["q10_return"] <= values["q50_return"] <= values["q90_return"]:
        raise ValueError("model return quantiles are not ordered")
    probabilities = sum(values[name] for name in ("down_probability", "flat_probability", "up_probability"))
    if abs(probabilities - 1.0) > 1e-4:
        raise ValueError("direction probabilities do not sum to one")
    masks = {key.removesuffix("_available"): bool(row[key]) for key in panel if key.endswith("_available")}
    ages = {key.removesuffix("_age_seconds"): float(row[key]) for key in panel if key.endswith("_age_seconds") and pd.notna(row[key])}
    return {
        **values,
        "decision_at": decision_at,
        "horizon_end": decision_at + timedelta(hours=1),
        "reference_price": price,
        "model_version": str(manifest["run_id"]),
        "data_version": str(manifest.get("data_version", "unknown")),
        "feature_version": str(manifest["feature_version"]),
        "source_masks": masks,
        "source_age_seconds": ages,
    }


def run_worker() -> None:
    logging.basicConfig(
        level=os.getenv("TRADING_MODEL_LOG_LEVEL", "INFO").upper(),
        format="%(asctime)s %(levelname)s %(name)s %(message)s",
    )
    interval = max(15, int(os.getenv("TRADING_MODEL_POLL_SECONDS", "30")))
    predictor = None
    config = None
    manifest = None
    while True:
        if predictor is None:
            try:
                run_dir = Path(_required_env("TRADING_MODEL_RUN_DIR"))
                if not run_dir.is_dir():
                    raise RuntimeError("configured model run directory is unavailable")
                config, manifest = _runtime_config(run_dir)
                if manifest.get("feature_version") != "xauusd-asof-v1":
                    raise RuntimeError("unsupported feature version in model run")
                predictor = SavedMultimodalPredictor(run_dir)
                logger.info("loaded evaluated XAUUSD run %s; feature version %s", manifest["run_id"], manifest["feature_version"])
            except Exception as exc:
                logger.warning("model unavailable: %s", type(exc).__name__)
                time.sleep(interval)
                continue
        try:
            for decision_at in _unprocessed_decisions(config, manifest["run_id"]):
                row = _forecast_for(config, predictor, manifest, decision_at)
                _insert_forecast(config.postgres_url, row)
                logger.info("stored XAUUSD forecast at %s for model %s", decision_at.isoformat(), manifest["run_id"])
        except Exception as exc:
            # Credentials and raw input data are deliberately excluded from worker logs.
            logger.error("forecast cycle unavailable: %s", type(exc).__name__)
        time.sleep(interval)
