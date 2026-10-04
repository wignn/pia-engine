from __future__ import annotations

import re
from datetime import timedelta
from typing import Any

import numpy as np
import pandas as pd

SOURCE_FRESHNESS = {
    "5m": timedelta(minutes=10),
    "15m": timedelta(minutes=30),
    "1h": timedelta(hours=2),
    "news": timedelta(hours=8),
    "social": timedelta(hours=2),
    "geosignals": timedelta(days=2),
    "macro": timedelta(days=45),
    "options": timedelta(days=1),
    "calendar": timedelta(days=7),
    "cot": timedelta(days=8),
    "fear_greed": timedelta(days=10),
    "central_bank": timedelta(days=30),
}


def _polarity(value: Any) -> float:
    label = str(value or "").lower()
    if label in {"positive", "bullish", "pos"}:
        return 1.0
    if label in {"negative", "bearish", "neg"}:
        return -1.0
    return 0.0


def _impact(value: Any) -> float:
    return {"low": 1.0, "medium": 2.0, "high": 3.0}.get(str(value or "").lower(), 0.0)


def _is_gold_related(row: pd.Series) -> bool:
    text = " ".join(str(row.get(column) or "") for column in ("title", "summary", "content", "text", "category"))
    text = text.lower()
    assets = row.get("affected_assets") or []
    asset_text = " ".join(map(str, assets)).lower() if isinstance(assets, (list, tuple, set)) else str(assets).lower()
    return any(term in text or term in asset_text for term in ("gold", "xau", "bullion", "precious metal"))


def _event_windows(decisions: pd.Series, events: pd.DataFrame, freshness: timedelta) -> list[pd.DataFrame]:
    if events.empty or "available_at" not in events:
        return [events.iloc[0:0] for _ in decisions]
    data = events.copy()
    data["available_at"] = pd.to_datetime(data["available_at"], utc=True, errors="coerce")
    data = data.dropna(subset=["available_at"]).sort_values("available_at").reset_index(drop=True)
    times = data["available_at"].astype("int64").to_numpy()
    age_ns = int(freshness.total_seconds() * 1_000_000_000)
    result = []
    for decision in pd.to_datetime(decisions, utc=True):
        stamp = decision.value
        right = int(np.searchsorted(times, stamp, side="right"))
        left = int(np.searchsorted(times, stamp - age_ns, side="left"))
        result.append(data.iloc[left:right])
    return result


def _aggregate_events(
    panel: pd.DataFrame,
    name: str,
    events: pd.DataFrame,
    freshness: timedelta,
) -> None:
    windows = _event_windows(panel["decision_at"], events, freshness)
    counts, relevant, ages, available = [], [], [], []
    sentiment, impact, engagement, surprise = [], [], [], []
    for decision, rows in zip(panel["decision_at"], windows):
        available.append(not rows.empty)
        counts.append(len(rows))
        ages.append((decision - rows["available_at"].max()).total_seconds() if not rows.empty else np.nan)
        if name in {"news", "social", "geosignals", "central_bank"}:
            relevant.append(sum(_is_gold_related(row) for _, row in rows.iterrows()))
        if name == "news":
            sentiment.append(rows["sentiment"].map(_polarity).mean() if not rows.empty else np.nan)
            impact.append(rows["impact_level"].map(_impact).mean() if not rows.empty else np.nan)
        elif name == "geosignals":
            sentiment.append(pd.to_numeric(rows.get("sentiment_score"), errors="coerce").mean() if not rows.empty else np.nan)
            impact.append(pd.to_numeric(rows.get("severity_score"), errors="coerce").mean() if not rows.empty else np.nan)
        elif name == "social":
            columns = [column for column in ("reply_count", "retweet_count", "like_count", "quote_count") if column in rows]
            values = rows[columns].apply(pd.to_numeric, errors="coerce").fillna(0).sum(axis=1) if columns and not rows.empty else pd.Series(dtype=float)
            engagement.append(values.mean() if len(values) else np.nan)
        elif name == "central_bank":
            impact.append(pd.to_numeric(rows.get("confidence"), errors="coerce").mean() if not rows.empty else np.nan)
        elif name == "calendar":
            if not rows.empty:
                actual = pd.to_numeric(rows["actual"], errors="coerce")
                forecast = pd.to_numeric(rows["forecast"], errors="coerce")
                surprise.append((actual - forecast).mean())
                impact.append(rows["impact"].map(_impact).mean())
            else:
                surprise.append(np.nan)
                impact.append(np.nan)
    panel[f"{name}_available"] = available
    panel[f"{name}_count"] = counts
    panel[f"{name}_age_seconds"] = ages
    if relevant:
        panel[f"{name}_gold_related_count"] = relevant
    if sentiment:
        panel[f"{name}_sentiment_mean"] = sentiment
    if impact:
        panel[f"{name}_impact_mean"] = impact
    if engagement:
        panel["social_engagement_mean"] = engagement
    if surprise:
        panel["calendar_surprise_mean"] = surprise


def _add_latest_numeric(
    panel: pd.DataFrame,
    name: str,
    rows: pd.DataFrame,
    freshness: timedelta,
    key_column: str | None = None,
    value_columns: tuple[str, ...] = (),
) -> None:
    if rows.empty or "available_at" not in rows:
        panel[f"{name}_available"] = False
        panel[f"{name}_age_seconds"] = np.nan
        return
    data = rows.copy()
    data["available_at"] = pd.to_datetime(data["available_at"], utc=True, errors="coerce")
    data = data.dropna(subset=["available_at"])
    sort_columns = ["available_at"]
    if "report_date" in data:
        sort_columns.append("report_date")
    if "id" in data:
        sort_columns.append("id")
    data = data.sort_values(sort_columns)
    if key_column is None:
        groups = [(name, data)]
    else:
        groups = list(data.groupby(key_column, dropna=False))
    decision_values = pd.to_datetime(panel["decision_at"], utc=True)
    source_any = np.zeros(len(panel), dtype=bool)
    source_age = np.full(len(panel), np.nan)
    for key, group in groups:
        stamps = group["available_at"].astype("int64").to_numpy()
        prefix = re.sub(r"[^A-Za-z0-9]+", "_", str(key)).strip("_").lower()[:80]
        prefix = f"{name}_{prefix}" if key_column else name
        age_values = np.full(len(panel), np.nan)
        for index, decision in enumerate(decision_values):
            right = int(np.searchsorted(stamps, decision.value, side="right"))
            if right == 0:
                continue
            row = group.iloc[right - 1]
            age = (decision - row["available_at"]).total_seconds()
            if age < 0 or age > freshness.total_seconds():
                continue
            age_values[index] = age
            for column in value_columns:
                if column in row.index:
                    panel.loc[panel.index[index], f"{prefix}_{column}"] = row[column]
        available = np.isfinite(age_values)
        source_any |= available
        source_age[available] = np.fmin(source_age[available], age_values[available]) if np.isfinite(source_age[available]).any() else age_values[available]
        panel[f"{prefix}_available"] = available
        panel[f"{prefix}_age_seconds"] = age_values
    panel[f"{name}_available"] = source_any
    panel[f"{name}_age_seconds"] = source_age


def _add_macro_features(panel: pd.DataFrame, rows: pd.DataFrame, freshness: timedelta) -> None:
    if rows.empty:
        panel["macro_available"] = False
        panel["macro_age_seconds"] = np.nan
        return
    data = rows.copy()
    data["available_at"] = pd.to_datetime(data["available_at"], utc=True, errors="coerce")
    data["observation_date"] = pd.to_datetime(data["observation_date"], utc=True, errors="coerce")
    data = data.dropna(subset=["available_at", "observation_date"])
    sort_columns = ["available_at"] + (["id"] if "id" in data else [])
    data = data.sort_values(sort_columns)
    source_any = np.zeros(len(panel), dtype=bool)
    source_age = np.full(len(panel), np.nan)
    decisions = pd.to_datetime(panel["decision_at"], utc=True)
    for series_key, group in data.groupby("series_key", dropna=False):
        available_ns = group["available_at"].astype("int64").to_numpy()
        prefix = re.sub(r"[^A-Za-z0-9]+", "_", str(series_key)).strip("_").lower()[:80]
        age_values = np.full(len(panel), np.nan)
        values = np.full(len(panel), np.nan)
        for index, decision in enumerate(decisions):
            right = int(np.searchsorted(available_ns, decision.value, side="right"))
            if right == 0:
                continue
            visible = group.iloc[:right].drop_duplicates("observation_date", keep="last")
            row = visible.loc[visible["observation_date"].idxmax()]
            age = (decision - row["available_at"]).total_seconds()
            if age < 0 or age > freshness.total_seconds():
                continue
            values[index] = float(row["value"])
            age_values[index] = age
        available = np.isfinite(age_values)
        source_any |= available
        source_age[available] = np.fmin(source_age[available], age_values[available]) if np.isfinite(source_age[available]).any() else age_values[available]
        panel[f"macro_{prefix}_value"] = values
        panel[f"macro_{prefix}_available"] = available
        panel[f"macro_{prefix}_age_seconds"] = age_values
    panel["macro_available"] = source_any
    panel["macro_age_seconds"] = source_age


def _add_market_features(panel: pd.DataFrame, market: pd.DataFrame, freshness: dict[str, timedelta]) -> None:
    if market.empty:
        return
    data = market.copy()
    data["time"] = pd.to_datetime(data["time"], utc=True)
    minutes = {"5m": 5, "15m": 15, "1h": 60}
    data["close_at"] = data["time"] + data["resolution"].map(lambda value: pd.Timedelta(minutes=minutes[value]))
    data = data.sort_values("close_at")
    for (symbol, resolution), source in data.groupby(["symbol", "resolution"]):
        source = source.sort_values("close_at").copy()
        source["log_return"] = np.log(source["close"].astype(float)).diff()
        source["range_fraction"] = (source["high"] - source["low"]) / source["close"].replace(0, np.nan)
        source["realized_vol_4"] = source["log_return"].rolling(4, min_periods=4).std()
        source["realized_vol_16"] = source["log_return"].rolling(16, min_periods=16).std()
        prefix = re.sub(r"[^A-Za-z0-9]+", "_", str(symbol)).strip("_").lower()
        prefix = f"{prefix}_{resolution}"
        right = source[["close_at", "log_return", "range_fraction", "realized_vol_4", "realized_vol_16", "volume"]].rename(
            columns={"close_at": "source_at", "log_return": f"{prefix}_log_return", "range_fraction": f"{prefix}_range_fraction", "volume": f"{prefix}_volume"}
        )
        right = right.rename(
            columns={
                "realized_vol_4": f"{prefix}_realized_vol_4",
                "realized_vol_16": f"{prefix}_realized_vol_16",
            }
        )
        merged = pd.merge_asof(
            panel[["decision_at"]].sort_values("decision_at"),
            right.sort_values("source_at"),
            left_on="decision_at",
            right_on="source_at",
            direction="backward",
            tolerance=pd.Timedelta(freshness[resolution]),
        )
        age_column = f"{prefix}_age_seconds"
        panel[f"{prefix}_available"] = merged["source_at"].notna().to_numpy()
        panel[age_column] = (merged["decision_at"] - merged["source_at"]).dt.total_seconds().to_numpy()
        for column in (
            f"{prefix}_log_return",
            f"{prefix}_range_fraction",
            f"{prefix}_realized_vol_4",
            f"{prefix}_realized_vol_16",
            f"{prefix}_volume",
        ):
            panel[column] = merged[column].to_numpy()


def _add_cot(panel: pd.DataFrame, rows: pd.DataFrame, freshness: timedelta) -> None:
    if rows.empty:
        panel["cot_available"] = False
        panel["cot_age_seconds"] = np.nan
        return
    gold = rows[rows["market_name"].str.contains("gold", case=False, na=False)].copy()
    if gold.empty:
        _add_latest_numeric(panel, "cot", gold, freshness)
        return
    gold["net_noncommercial"] = pd.to_numeric(gold["noncommercial_long"], errors="coerce") - pd.to_numeric(
        gold["noncommercial_short"], errors="coerce"
    )
    gold["net_noncommercial_oi"] = gold["net_noncommercial"] / pd.to_numeric(gold["open_interest"], errors="coerce").replace(0, np.nan)
    _add_latest_numeric(panel, "cot", gold, freshness, "report_type", ("net_noncommercial", "net_noncommercial_oi", "open_interest"))


def build_asof_panel(
    market: pd.DataFrame,
    side_channels: dict[str, pd.DataFrame],
    freshness: dict[str, timedelta] | None = None,
) -> pd.DataFrame:
    limits = {**SOURCE_FRESHNESS, **(freshness or {})}
    if market.empty:
        return pd.DataFrame()
    candles = market.copy()
    candles["time"] = pd.to_datetime(candles["time"], utc=True)
    gold = candles[(candles["symbol"].str.upper() == "XAUUSD") & (candles["resolution"] == "15m")].copy()
    gold = gold.sort_values("time").reset_index(drop=True)
    if gold.empty:
        return pd.DataFrame()
    gold["decision_at"] = gold["time"] + pd.Timedelta(minutes=15)
    gold["target_end"] = gold["decision_at"] + pd.Timedelta(hours=1)
    future_close = gold["close"].shift(-4)
    future_time = gold["time"].shift(-4)
    contiguous = future_time.eq(gold["time"] + pd.Timedelta(hours=1))
    gold["target_log_return_1h"] = np.where(contiguous, np.log(future_close / gold["close"]), np.nan)
    panel = gold[["decision_at", "target_end", "open", "high", "low", "close", "volume", "target_log_return_1h"]].copy()
    panel = panel.rename(columns={column: f"xau_15m_{column}" for column in ("open", "high", "low", "close", "volume")})
    panel = panel.sort_values("decision_at").reset_index(drop=True)
    hour = panel["decision_at"].dt.hour + panel["decision_at"].dt.minute / 60.0
    panel["utc_hour_sin"] = np.sin(2 * np.pi * hour / 24.0)
    panel["utc_hour_cos"] = np.cos(2 * np.pi * hour / 24.0)
    panel["day_of_week"] = panel["decision_at"].dt.dayofweek.astype("int8")
    _add_market_features(panel, candles, limits)

    for name in ("news", "social", "geosignals", "calendar", "central_bank"):
        _aggregate_events(panel, name, side_channels.get(name, pd.DataFrame()), limits[name])
    _add_macro_features(panel, side_channels.get("macro", pd.DataFrame()), limits["macro"])
    _add_latest_numeric(
        panel,
        "options",
        side_channels.get("options", pd.DataFrame()),
        limits["options"],
        "symbol",
        ("underlying_price", "put_call_ratio", "max_pain_strike", "total_open_interest", "total_volume", "total_gex", "iv_atm"),
    )
    _add_cot(panel, side_channels.get("cot", pd.DataFrame()), limits["cot"])
    _add_latest_numeric(panel, "fear_greed", side_channels.get("fear_greed", pd.DataFrame()), limits["fear_greed"], None, ("score",))

    contracts = side_channels.get("options_contracts", pd.DataFrame())
    if not contracts.empty:
        contracts = contracts.copy()
        contracts["available_at"] = pd.to_datetime(contracts["available_at"], utc=True)
        contracts["expiration_date"] = pd.to_datetime(contracts["expiration_date"], utc=True)
        contracts["days_to_expiry"] = (contracts["expiration_date"] - contracts["available_at"]).dt.total_seconds() / 86400
        contracts["notional_gamma"] = pd.to_numeric(contracts["gamma"], errors="coerce") * pd.to_numeric(
            contracts["open_interest"], errors="coerce"
        )
        by_snapshot = contracts.groupby("snapshot_id").agg(
            options_contract_count=("contract_symbol", "count"),
            options_contract_iv_mean=("implied_volatility", "mean"),
            options_contract_oi_sum=("open_interest", "sum"),
            options_contract_gamma_oi=("notional_gamma", "sum"),
        )
        snapshots = side_channels.get("option_chains", pd.DataFrame())
        if not snapshots.empty:
            snapshots = snapshots.merge(by_snapshot, left_on="id", right_index=True, how="left")
            _add_latest_numeric(
                panel,
                "options_chain",
                snapshots,
                limits["options"],
                "symbol",
                ("options_contract_count", "options_contract_iv_mean", "options_contract_oi_sum", "options_contract_gamma_oi"),
            )
        else:
            _add_latest_numeric(panel, "options_chain", pd.DataFrame(), limits["options"])
    else:
        _add_latest_numeric(panel, "options_chain", pd.DataFrame(), limits["options"])
    panel.replace([np.inf, -np.inf], np.nan, inplace=True)
    return panel


def coverage_report(panel: pd.DataFrame) -> dict[str, Any]:
    if panel.empty:
        return {"rows": 0, "sources": {}}
    sources: dict[str, Any] = {}
    for column in panel.columns:
        if not column.endswith("_available"):
            continue
        name = column.removesuffix("_available")
        age_column = f"{name}_age_seconds"
        ages = pd.to_numeric(panel[age_column], errors="coerce") if age_column in panel else pd.Series(dtype=float)
        available = panel[column].fillna(False).astype(bool)
        sources[name] = {
            "available_fraction": float(available.mean()),
            "available_rows": int(available.sum()),
            "first_available_at": str(panel.loc[available, "decision_at"].min()) if available.any() else None,
            "age_seconds_p50": float(ages.median()) if ages.notna().any() else None,
            "age_seconds_p95": float(ages.quantile(0.95)) if ages.notna().any() else None,
        }
    return {
        "rows": int(len(panel)),
        "decision_start": str(panel["decision_at"].min()),
        "decision_end": str(panel["decision_at"].max()),
        "target_rows": int(panel["target_log_return_1h"].notna().sum()),
        "sources": sources,
    }
