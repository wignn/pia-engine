from __future__ import annotations

import json
from pathlib import Path
from typing import Any

import numpy as np
import pandas as pd
from sklearn.dummy import DummyClassifier
from sklearn.ensemble import HistGradientBoostingClassifier, HistGradientBoostingRegressor

from .config import ResearchConfig

EvaluationReport = dict[str, Any]
TARGET = "target_log_return_1h"
CLASS_NAMES = ("down", "flat", "up")
RETURN_SCALE = 10_000.0


def _aligned_probabilities(estimator: Any, values: np.ndarray, temperature: float = 1.0) -> np.ndarray:
    raw = estimator.predict_proba(values)
    result = np.full((len(values), 3), 1e-8, dtype=float)
    result[:, np.asarray(estimator.classes_, dtype=int)] = raw
    logits = np.log(np.clip(result, 1e-8, 1.0)) / max(0.1, temperature)
    logits -= logits.max(axis=1, keepdims=True)
    probabilities = np.exp(logits)
    return probabilities / probabilities.sum(axis=1, keepdims=True)


def _fit_temperature(logits: np.ndarray, labels: np.ndarray) -> float:
    if len(labels) < 8:
        return 1.0
    best_temperature, best_loss = 1.0, float("inf")
    for temperature in np.linspace(0.5, 3.0, 51):
        scaled = logits / temperature
        scaled -= scaled.max(axis=1, keepdims=True)
        probabilities = np.exp(scaled)
        probabilities /= probabilities.sum(axis=1, keepdims=True)
        loss = -np.log(np.clip(probabilities[np.arange(len(labels)), labels.astype(int)], 1e-8, 1.0)).mean()
        if loss < best_loss:
            best_temperature, best_loss = float(temperature), float(loss)
    return best_temperature


def _base_scores(predictions: pd.DataFrame) -> dict[str, Any]:
    if predictions.empty:
        return {"rows": 0}
    probabilities = predictions[["down_probability", "flat_probability", "up_probability"]].to_numpy(float)
    labels = predictions["actual_class"].to_numpy(int)
    one_hot = np.eye(3)[labels]
    truth_probability = np.clip(probabilities[np.arange(len(labels)), labels], 1e-8, 1.0)
    predicted = probabilities.argmax(axis=1)
    confidence = probabilities.max(axis=1)
    correct = (predicted == labels).astype(float)
    ece = 0.0
    for lower in np.linspace(0.0, 0.9, 10):
        selected = (confidence >= lower) & (confidence < lower + 0.1)
        if selected.any():
            ece += selected.mean() * abs(confidence[selected].mean() - correct[selected].mean())
    actual = predictions["actual_return_bps"].to_numpy(float)
    expected = predictions["expected_return_bps"].to_numpy(float)
    quantiles = predictions[["q10_return_bps", "q50_return_bps", "q90_return_bps"]].to_numpy(float)
    q_values = np.asarray([0.1, 0.5, 0.9])
    errors = actual[:, None] - quantiles
    pinball = np.maximum(q_values * errors, (q_values - 1.0) * errors).mean()
    return {
        "rows": int(len(predictions)),
        "direction_accuracy": float((predicted == labels).mean()),
        "brier_score": float(np.square(probabilities - one_hot).sum(axis=1).mean() / 3.0),
        "log_loss": float(-np.log(truth_probability).mean()),
        "expected_calibration_error": float(ece),
        "return_mae_bps": float(np.abs(actual - expected).mean()),
        "return_rmse_bps": float(np.sqrt(np.square(actual - expected).mean())),
        "quantile_pinball_bps": float(pinball),
    }


def _cost_scores(predictions: pd.DataFrame, settings: dict[str, Any]) -> dict[str, Any]:
    probability_threshold = float(settings.get("trade_probability_threshold", 0.45))
    up = predictions["up_probability"].to_numpy(float)
    down = predictions["down_probability"].to_numpy(float)
    signals = np.where((up >= probability_threshold) & (up > down), 1.0, 0.0)
    signals = np.where((down >= probability_threshold) & (down > up), -1.0, signals)
    actual = predictions["actual_return_bps"].to_numpy(float)
    round_trip = float(settings.get("spread_bps", 0.0)) + 2 * float(settings.get("commission_per_side_bps", 0.0)) + 2 * float(
        settings.get("slippage_per_side_bps", 0.0)
    )
    scenarios = settings.get("cost_sensitivity_bps", [round_trip])
    cost_results = []
    for cost in scenarios:
        net = signals * actual - np.where(signals != 0, float(cost), 0.0)
        equity = np.cumsum(net)
        drawdown = equity - np.maximum.accumulate(np.r_[0.0, equity])[:-1]
        turns = np.abs(np.diff(np.r_[0.0, signals]))
        cost_results.append(
            {
                "round_trip_cost_bps": float(cost),
                "net_total_bps": float(net.sum()),
                "net_mean_bps_per_decision": float(net.mean()),
                "max_drawdown_bps": float(-drawdown.min()) if len(drawdown) else 0.0,
                "turnover": float(turns.sum()),
                "trades": int(np.count_nonzero(turns)),
                "exposure_fraction": float(np.abs(signals).mean()),
            }
        )
    return {
        "cost_assumptions": {
        "spread_bps": float(settings.get("spread_bps", 0.0)),
        "commission_per_side_bps": float(settings.get("commission_per_side_bps", 0.0)),
        "slippage_per_side_bps": float(settings.get("slippage_per_side_bps", 0.0)),
        "round_trip_cost_bps": round_trip,
        },
        "net_scenarios": cost_results,
    }


def score_predictions(predictions: pd.DataFrame, costs: dict[str, Any] | None = None) -> dict[str, Any]:
    result = _base_scores(predictions)
    if predictions.empty:
        return result
    settings = costs or {}
    result.update(_cost_scores(predictions, settings))
    if "decision_at" in predictions:
        dates = pd.to_datetime(predictions["decision_at"], utc=True)
        period = predictions.copy()
        period["period"] = dates.dt.strftime("%Y-%m")
        result["by_period"] = {
            key: {**_base_scores(group), **_cost_scores(group, settings)} for key, group in period.groupby("period", sort=True)
        }
        if "volatility_regime" in predictions:
            result["by_regime"] = {
                key: {**_base_scores(group), **_cost_scores(group, settings)}
                for key, group in predictions.groupby("volatility_regime", sort=True)
            }
    return result


def _labels(panel: pd.DataFrame, threshold: float) -> tuple[np.ndarray, np.ndarray]:
    returns = pd.to_numeric(panel[TARGET], errors="coerce").to_numpy(float)
    classes = np.where(returns > threshold, 2, np.where(returns < -threshold, 0, 1))
    return classes, returns * RETURN_SCALE


def _prediction_rows(
    panel: pd.DataFrame,
    indices: np.ndarray,
    labels: np.ndarray,
    returns_bps: np.ndarray,
    probabilities: np.ndarray,
    expected_bps: np.ndarray,
    residual_quantiles: np.ndarray,
    model_name: str,
) -> pd.DataFrame:
    quantiles = expected_bps[:, None] + residual_quantiles[None, :]
    volatility_column = "xau_15m_realized_vol_16"
    volatility = pd.to_numeric(panel.iloc[indices].get(volatility_column, pd.Series(np.nan, index=indices)), errors="coerce").to_numpy(float)
    regime_cut = np.nanmedian(volatility) if np.isfinite(volatility).any() else np.nan
    regimes = np.where(np.isfinite(volatility) & (volatility >= regime_cut), "high_volatility", "low_or_missing_volatility")
    return pd.DataFrame(
        {
            "model": model_name,
            "decision_at": panel.iloc[indices]["decision_at"].to_numpy(),
            "actual_class": labels[indices],
            "actual_return_bps": returns_bps[indices],
            "down_probability": probabilities[:, 0],
            "flat_probability": probabilities[:, 1],
            "up_probability": probabilities[:, 2],
            "expected_return_bps": expected_bps,
            "q10_return_bps": quantiles[:, 0],
            "q50_return_bps": quantiles[:, 1],
            "q90_return_bps": quantiles[:, 2],
            "volatility_regime": regimes,
        }
    )


def _evaluate_estimator(
    panel: pd.DataFrame,
    core_indices: np.ndarray,
    calibration_indices: np.ndarray,
    evaluation_indices: np.ndarray,
    feature_columns: list[str],
    labels: np.ndarray,
    returns_bps: np.ndarray,
    config: ResearchConfig,
    model_name: str,
) -> pd.DataFrame:
    from .train import _matrix, _standardize

    matrix, _transform = _standardize(_matrix(panel, feature_columns), core_indices)
    classifier: Any = (
        HistGradientBoostingClassifier(max_iter=100, max_leaf_nodes=15, l2_regularization=1.0, random_state=config.seed)
        if len(np.unique(labels[core_indices])) > 1
        else DummyClassifier(strategy="prior")
    )
    regressor = HistGradientBoostingRegressor(max_iter=100, max_leaf_nodes=15, l2_regularization=1.0, random_state=config.seed)
    classifier.fit(matrix[core_indices], labels[core_indices])
    regressor.fit(matrix[core_indices], returns_bps[core_indices])
    calibration_probabilities = _aligned_probabilities(classifier, matrix[calibration_indices])
    temperature = _fit_temperature(np.log(np.clip(calibration_probabilities, 1e-8, 1.0)), labels[calibration_indices])
    probabilities = _aligned_probabilities(classifier, matrix[evaluation_indices], temperature)
    calibration_residuals = returns_bps[calibration_indices] - regressor.predict(matrix[calibration_indices])
    residual_quantiles = np.quantile(calibration_residuals, [0.1, 0.5, 0.9]) if len(calibration_residuals) else np.zeros(3)
    expected = regressor.predict(matrix[evaluation_indices])
    return _prediction_rows(panel, evaluation_indices, labels, returns_bps, probabilities, expected, residual_quantiles, model_name)


def _feature_sets(panel: pd.DataFrame) -> dict[str, list[str]]:
    from .train import _columns

    numeric = [
        column
        for column in panel.select_dtypes(include=[np.number, "bool"]).columns
        if column != TARGET and not column.startswith("target_")
    ]
    price, groups, _ = _columns(panel)
    sets = {"price_only": price}
    for name, columns in groups.items():
        combined = list(dict.fromkeys(price + columns))
        if combined:
            sets[f"price_plus_{name}"] = combined
    sets["full_tabular"] = numeric
    return {name: columns for name, columns in sets.items() if columns}


def _run_fold(
    panel: pd.DataFrame,
    development: np.ndarray,
    start: int,
    end: int,
    labels: np.ndarray,
    returns_bps: np.ndarray,
    config: ResearchConfig,
    feature_sets: dict[str, list[str]],
    fold_name: str,
) -> list[pd.DataFrame]:
    evaluation_indices = development[start:end]
    fit_pool = development[: max(0, start - 4)]
    calibration_start = int(len(fit_pool) * 0.8)
    core_indices = fit_pool[: max(0, calibration_start - 4)]
    calibration_indices = fit_pool[calibration_start:]
    if len(core_indices) < 30 or len(calibration_indices) < 8 or not len(evaluation_indices):
        return []
    predictions = []
    for name, columns in feature_sets.items():
        output = _evaluate_estimator(
            panel,
            core_indices,
            calibration_indices,
            evaluation_indices,
            columns,
            labels,
            returns_bps,
            config,
            name,
        )
        output["fold"] = fold_name
        predictions.append(output)

    no_change_probabilities = np.zeros((len(evaluation_indices), 3))
    no_change_probabilities[:, 1] = 1.0
    no_change = _prediction_rows(
        panel,
        evaluation_indices,
        labels,
        returns_bps,
        no_change_probabilities,
        np.zeros(len(evaluation_indices)),
        np.zeros(3),
        "no_change",
    )
    no_change["fold"] = fold_name
    predictions.append(no_change)
    return predictions


def walk_forward(panel: pd.DataFrame, config: ResearchConfig) -> EvaluationReport:
    if panel.empty or TARGET not in panel:
        return {"status": "insufficient_data", "reason": "empty panel or missing target", "folds": [], "models": {}}
    panel = panel.sort_values("decision_at").reset_index(drop=True)
    labels, returns_bps = _labels(panel, config.neutral_return_threshold)
    valid = np.flatnonzero(np.isfinite(returns_bps))
    cutoff = int(len(valid) * 0.8)
    development, final = valid[:cutoff], valid[cutoff:]
    if len(development) < 80 or len(final) < 8:
        return {
            "status": "exploratory_insufficient_data",
            "reason": f"need >=80 development and >=8 final labeled rows; got {len(development)} and {len(final)}",
            "development_rows": len(development),
            "final_rows": len(final),
            "folds": [],
            "models": {},
        }
    feature_sets = _feature_sets(panel)
    boundaries = np.linspace(max(80, int(len(development) * 0.4)), len(development), 5, dtype=int)
    predictions = []
    folds = []
    for number, (start, end) in enumerate(zip(boundaries[:-1], boundaries[1:]), start=1):
        fold_name = f"fold_{number}"
        folds.append(
            {
                "fold": fold_name,
                "train_end": str(panel.iloc[development[max(0, start - 5)]]["decision_at"]),
                "validation_start": str(panel.iloc[development[start]]["decision_at"]),
                "validation_end": str(panel.iloc[development[end - 1]]["decision_at"]),
                "purge_candles": 4,
            }
        )
        predictions.extend(
            _run_fold(panel, development, start, end, labels, returns_bps, config, feature_sets, fold_name)
        )
    prediction_frame = pd.concat(predictions, ignore_index=True) if predictions else pd.DataFrame()
    cost_config = {
        "spread_bps": config.spread_bps,
        "commission_per_side_bps": config.commission_per_side_bps,
        "slippage_per_side_bps": config.slippage_per_side_bps,
        "cost_sensitivity_bps": config.cost_sensitivity_bps,
        "trade_probability_threshold": config.trade_probability_threshold,
    }
    model_scores = {
        name: score_predictions(group, cost_config)
        for name, group in prediction_frame.groupby("model", sort=True)
    } if not prediction_frame.empty else {}

    # The untouched final interval is scored only after the fixed feature and split protocol above.
    final_oos = []
    fit_pool = development[:-4]
    calibration_start = int(len(fit_pool) * 0.8)
    core = fit_pool[: max(0, calibration_start - 4)]
    calibration = fit_pool[calibration_start:]
    if len(core) >= 30 and len(calibration) >= 8 and len(final):
        for name, columns in feature_sets.items():
            final_oos.append(
                _evaluate_estimator(panel, core, calibration, final, columns, labels, returns_bps, config, name)
            )
        no_change_probabilities = np.zeros((len(final), 3))
        no_change_probabilities[:, 1] = 1.0
        no_change = _prediction_rows(panel, final, labels, returns_bps, no_change_probabilities, np.zeros(len(final)), np.zeros(3), "no_change")
        final_oos.append(no_change)
    final_frame = pd.concat(final_oos, ignore_index=True) if final_oos else pd.DataFrame()
    final_scores = {
        name: score_predictions(group, cost_config) for name, group in final_frame.groupby("model", sort=True)
    } if not final_frame.empty else {}
    return {
        "status": "exploratory",
        "note": "Short local history; metrics are not validated alpha and must not be used to tune repeatedly against the final interval.",
        "data_rows": int(len(panel)),
        "development_rows": int(len(development)),
        "untouched_final_rows": int(len(final)),
        "untouched_final_start": str(panel.iloc[final[0]]["decision_at"]) if len(final) else None,
        "trial_count": len(folds) * len(feature_sets) + (len(feature_sets) if final_scores else 0),
        "trained_candidate_count": 3,
        "cost_config": cost_config,
        "folds": folds,
        "walk_forward_models": model_scores,
        "final_holdout": {"rows": len(final_frame), "models": final_scores},
        "modality_ablation_models": list(feature_sets),
    }


def _apply_transform(panel: pd.DataFrame, columns: list[str], transform: dict[str, list[float]]) -> np.ndarray:
    values = np.full((len(panel), len(columns)), np.nan, dtype=np.float64)
    for index, column in enumerate(columns):
        if column in panel:
            values[:, index] = pd.to_numeric(panel[column], errors="coerce").to_numpy(dtype=float)
    median = np.asarray(transform["median"], dtype=float)
    mean = np.asarray(transform["mean"], dtype=float)
    scale = np.asarray(transform["scale"], dtype=float)
    values = np.where(np.isfinite(values), values, median)
    return ((values - mean) / scale).astype(np.float32)


def _predict_saved_neural(panel: pd.DataFrame, path: Path) -> pd.DataFrame:
    import torch

    from .model import MultimodalForecaster
    from .train import _mask_matrix, _sequences

    checkpoint = torch.load(path, map_location="cpu", weights_only=True)
    price = _apply_transform(panel, checkpoint["price_columns"], checkpoint["price_transform"])
    sequences = _sequences(price, int(checkpoint["sequence_length"]))
    modalities, masks = {}, {}
    for name, columns in checkpoint["modality_columns"].items():
        modalities[name] = _apply_transform(panel, columns, checkpoint["modality_transforms"][name])
        masks[name] = _mask_matrix(panel, checkpoint["mask_columns"].get(name, []))
    model = MultimodalForecaster(
        price_features=sequences.shape[-1],
        modality_dims={name: values.shape[-1] for name, values in modalities.items()},
        quantiles=tuple(checkpoint["quantiles"]),
    )
    model.load_state_dict(checkpoint["state_dict"])
    model.eval()
    rows = []
    with torch.inference_mode():
        for start in range(0, len(panel), 128):
            stop = min(start + 128, len(panel))
            batch = {
                "price_seq": torch.tensor(sequences[start:stop], dtype=torch.float32),
                "modalities": {name: torch.tensor(values[start:stop], dtype=torch.float32) for name, values in modalities.items()},
                "masks": {name: torch.tensor(values[start:stop], dtype=torch.bool) for name, values in masks.items()},
            }
            output = model(batch)
            logits = output["direction_logits"].cpu().numpy() / float(checkpoint.get("calibration_temperature", 1.0))
            logits -= logits.max(axis=1, keepdims=True)
            probabilities = np.exp(logits)
            probabilities /= probabilities.sum(axis=1, keepdims=True)
            quantiles = output["return_quantiles"].cpu().numpy()
            uncertainty = output["uncertainty"].cpu().numpy()
            for index in range(stop - start):
                q = quantiles[index] / float(checkpoint["return_scale"])
                rows.append(
                    {
                        "down_probability": probabilities[index, 0],
                        "flat_probability": probabilities[index, 1],
                        "up_probability": probabilities[index, 2],
                        "expected_return": q[1],
                        "q10_return": q[0],
                        "q50_return": q[1],
                        "q90_return": q[2],
                        "uncertainty": uncertainty[index] / float(checkpoint["return_scale"]),
                    }
                )
    return pd.DataFrame(rows)


def predict_artifact(panel: pd.DataFrame, run_dir: Path, name: str) -> pd.DataFrame:
    manifest = json.loads((run_dir / "run.json").read_text())
    artifact_name = manifest["artifacts"].get(name)
    if not artifact_name:
        raise ValueError(f"run has no {name} artifact")
    path = run_dir / artifact_name
    if name == "multimodal" or name == "price_only":
        return _predict_saved_neural(panel, path)
    if name == "no_change":
        result = pd.DataFrame(index=panel.index)
        result["down_probability"] = 0.0
        result["flat_probability"] = 1.0
        result["up_probability"] = 0.0
        result["expected_return"] = 0.0
        result["q10_return"] = 0.0
        result["q50_return"] = 0.0
        result["q90_return"] = 0.0
        result["uncertainty"] = 0.0
        return result
    if name == "tabular":
        import joblib

        saved = joblib.load(path)
        values = _apply_transform(panel, saved["columns"], saved["transform"])
        probabilities = _aligned_probabilities(saved["classifier"], values, saved["calibration_temperature"])
        expected = saved["regressor"].predict(values) / RETURN_SCALE
        offsets = np.asarray(saved["residual_quantiles"], dtype=float) / RETURN_SCALE
        quantiles = expected[:, None] + offsets[None, :]
        return pd.DataFrame(
            {
                "down_probability": probabilities[:, 0],
                "flat_probability": probabilities[:, 1],
                "up_probability": probabilities[:, 2],
                "expected_return": expected,
                "q10_return": quantiles[:, 0],
                "q50_return": quantiles[:, 1],
                "q90_return": quantiles[:, 2],
                "uncertainty": (quantiles[:, 2] - quantiles[:, 0]) / 2,
            }
        )
    raise ValueError(f"unsupported model artifact: {name}")


def evaluate_run(panel: pd.DataFrame, run_dir: Path, config: ResearchConfig) -> dict[str, Any]:
    manifest = json.loads((run_dir / "run.json").read_text())
    final_start = pd.to_datetime(manifest["data_window"]["untouched_final_start"], utc=True)
    final = panel[
        (pd.to_datetime(panel["decision_at"], utc=True) >= final_start) & panel[TARGET].notna()
    ].copy()
    labels, returns_bps = _labels(panel, config.neutral_return_threshold)
    indices = final.index.to_numpy(dtype=int)
    predictions = {}
    metrics = {}
    costs = {
        "spread_bps": config.spread_bps,
        "commission_per_side_bps": config.commission_per_side_bps,
        "slippage_per_side_bps": config.slippage_per_side_bps,
        "cost_sensitivity_bps": config.cost_sensitivity_bps,
        "trade_probability_threshold": config.trade_probability_threshold,
    }
    for name in ("no_change", "tabular", "price_only", "multimodal"):
        output = predict_artifact(panel, run_dir, name).iloc[indices].reset_index(drop=True)
        output["decision_at"] = final["decision_at"].reset_index(drop=True)
        output["actual_class"] = labels[indices]
        output["actual_return_bps"] = returns_bps[indices]
        output["expected_return_bps"] = output["expected_return"] * RETURN_SCALE
        for quantile in ("q10", "q50", "q90"):
            output[f"{quantile}_return_bps"] = output[f"{quantile}_return"] * RETURN_SCALE
        metrics[name] = score_predictions(output, costs)
        predictions[name] = output
    return {
        "status": "exploratory",
        "rows": len(final),
        "start": str(final["decision_at"].min()) if len(final) else None,
        "end": str(final["decision_at"].max()) if len(final) else None,
        "models": metrics,
        "predictions": predictions,
    }


def predict_asof(panel: pd.DataFrame, run_dir: Path, as_of: str) -> dict[str, Any]:
    decision_times = pd.to_datetime(panel["decision_at"], utc=True)
    cutoff = pd.to_datetime(as_of, utc=True)
    eligible = np.flatnonzero(decision_times <= cutoff)
    if not len(eligible):
        raise ValueError(f"no eligible XAUUSD decision row at or before {as_of}")
    index = int(eligible[-1])
    row = panel.iloc[[index]]
    outputs = predict_artifact(panel, run_dir, "multimodal").iloc[index]
    manifest = json.loads((run_dir / "run.json").read_text())
    masks = {column: bool(row.iloc[0][column]) for column in row if column.endswith("_available")}
    ages = {column: float(row.iloc[0][column]) for column in row if column.endswith("_age_seconds") and pd.notna(row.iloc[0][column])}
    return {
        "decision_at": str(row.iloc[0]["decision_at"]),
        "horizon_end": str(row.iloc[0]["target_end"]),
        "probabilities": {
            "down": float(outputs["down_probability"]),
            "flat": float(outputs["flat_probability"]),
            "up": float(outputs["up_probability"]),
        },
        "expected_return": float(outputs["expected_return"]),
        "return_quantiles": {
            "q10": float(outputs["q10_return"]),
            "q50": float(outputs["q50_return"]),
            "q90": float(outputs["q90_return"]),
        },
        "uncertainty": float(outputs["uncertainty"]),
        "source_masks": masks,
        "source_age_seconds": ages,
        "model_version": manifest["run_id"],
        "data_version": manifest.get("data_version"),
        "feature_version": manifest["feature_version"],
        "paper_only": True,
    }
