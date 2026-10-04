from __future__ import annotations

import json
import hashlib
import platform
import time
import uuid
from importlib import metadata
from pathlib import Path
from typing import Any

import numpy as np
import pandas as pd
import torch
from sklearn.ensemble import HistGradientBoostingClassifier, HistGradientBoostingRegressor
from sklearn.dummy import DummyClassifier

from .config import ResearchConfig
from .features import coverage_report
from .model import MultimodalForecaster

TARGET = "target_log_return_1h"
QUANTILES = (0.1, 0.5, 0.9)
MARKET_PREFIXES = ("xagusd_", "dxy_", "usdx_", "eurusd_", "usdjpy_", "spy_", "gld_", "vix_", "gc_f_")


def _standardize(values: np.ndarray, fit_rows: np.ndarray) -> tuple[np.ndarray, dict[str, list[float]]]:
    train = values[fit_rows]
    medians = np.zeros(values.shape[1], dtype=np.float64)
    for column in range(values.shape[1]):
        observed = train[np.isfinite(train[:, column]), column]
        if len(observed):
            medians[column] = np.median(observed)
    filled = np.where(np.isfinite(values), values, medians)
    train_filled = filled[fit_rows]
    means = train_filled.mean(axis=0)
    scales = train_filled.std(axis=0)
    scales = np.where(np.isfinite(scales) & (scales > 1e-8), scales, 1.0)
    normalized = ((filled - means) / scales).astype(np.float32)
    return normalized, {"median": medians.tolist(), "mean": means.tolist(), "scale": scales.tolist()}


def _columns(panel: pd.DataFrame) -> tuple[list[str], dict[str, list[str]], dict[str, list[str]]]:
    numeric = [
        column
        for column in panel.select_dtypes(include=[np.number, "bool"]).columns
        if column != TARGET and not column.startswith("target_")
    ]
    price = [column for column in numeric if column.startswith("xau_")]
    groups: dict[str, list[str]] = {
        "cross_market": [column for column in numeric if column.startswith(MARKET_PREFIXES)],
        "news_events": [
            column
            for column in numeric
            if column.startswith(("news_", "social_", "geosignals_", "calendar_", "central_bank_"))
            and not column.startswith("news_embedding_")
        ],
        "macro_position": [column for column in numeric if column.startswith(("macro_", "cot_", "fear_greed_"))],
        "options": [column for column in numeric if column.startswith("options_")],
        "text": [column for column in numeric if column.startswith("news_embedding_")],
        "chronos": [column for column in numeric if column.startswith("chronos_return_")],
    }
    masks = {
        "cross_market": [column for column in panel if column.endswith("_available") and column.startswith(MARKET_PREFIXES)],
        "news_events": [f"{name}_available" for name in ("news", "social", "geosignals", "calendar", "central_bank")],
        "macro_position": ["macro_available", "cot_available", "fear_greed_available"],
        "options": ["options_available", "options_chain_available"],
        "text": ["news_embedding_available"],
        "chronos": ["chronos_available"],
    }
    groups = {name: columns for name, columns in groups.items() if columns}
    return price, groups, masks


def _matrix(panel: pd.DataFrame, columns: list[str]) -> np.ndarray:
    if not columns:
        return np.empty((len(panel), 0), dtype=np.float32)
    return panel[columns].apply(pd.to_numeric, errors="coerce").to_numpy(dtype=np.float64)


def _temperature(logits: np.ndarray, labels: np.ndarray) -> float:
    if len(labels) < 8:
        return 1.0
    labels = labels.astype(int)
    best_temperature, best_loss = 1.0, float("inf")
    for temperature in np.linspace(0.5, 3.0, 51):
        scaled = logits / temperature
        scaled -= scaled.max(axis=1, keepdims=True)
        probabilities = np.exp(scaled)
        probabilities /= probabilities.sum(axis=1, keepdims=True)
        loss = -np.log(np.clip(probabilities[np.arange(len(labels)), labels], 1e-8, 1.0)).mean()
        if loss < best_loss:
            best_temperature, best_loss = float(temperature), float(loss)
    return best_temperature


def _mask_matrix(panel: pd.DataFrame, columns: list[str]) -> np.ndarray:
    present = [column for column in columns if column in panel]
    if not present:
        return np.zeros(len(panel), dtype=bool)
    return panel[present].fillna(False).astype(bool).any(axis=1).to_numpy()


def _sequences(values: np.ndarray, length: int) -> np.ndarray:
    if not len(values):
        return np.empty((0, length, values.shape[-1]), dtype=np.float32)
    result = np.empty((len(values), length, values.shape[-1]), dtype=np.float32)
    for index in range(len(values)):
        start = max(0, index - length + 1)
        window = values[start : index + 1]
        if len(window) < length:
            padding = np.repeat(window[:1], length - len(window), axis=0)
            window = np.concatenate([padding, window], axis=0)
        result[index] = window
    return result


def _chronos_features(panel: pd.DataFrame, config: ResearchConfig) -> tuple[pd.DataFrame, dict[str, Any]]:
    result = panel.copy()
    if not config.include_chronos:
        result["chronos_available"] = False
        return result, {"selected_backbone": "price_only", "reason": "Chronos disabled in config"}
    try:
        from chronos import Chronos2Pipeline

        device = "cuda" if torch.cuda.is_available() else "cpu"
        started = time.perf_counter()
        pipeline = Chronos2Pipeline.from_pretrained(config.model_id, device_map=device)
        price = np.log(pd.to_numeric(result["xau_15m_close"], errors="coerce").to_numpy(dtype=float))
        covariate_columns = [column for column in result if column.endswith("_log_return") and not column.startswith("xau_")]
        histories: list[dict[str, Any]] = []
        positions: list[int] = []
        for index in range(len(result)):
            start = max(0, index - 127)
            if index - start + 1 < 16 or not np.isfinite(price[start : index + 1]).all():
                continue
            item: dict[str, Any] = {"target": torch.tensor(price[start : index + 1], dtype=torch.float32)}
            if covariate_columns:
                item["past_covariates"] = {
                    column: torch.nan_to_num(
                        torch.tensor(pd.to_numeric(result[column].iloc[start : index + 1], errors="coerce").to_numpy(dtype=float), dtype=torch.float32)
                    )
                    for column in covariate_columns
                }
            histories.append(item)
            positions.append(index)

        columns = ("chronos_return_q10_bps", "chronos_return_q50_bps", "chronos_return_q90_bps")
        for column in columns:
            result[column] = np.nan
        result["chronos_available"] = False
        for batch_start in range(0, len(histories), max(1, config.batch_size)):
            batch = histories[batch_start : batch_start + config.batch_size]
            forecasts = pipeline.predict(batch, prediction_length=4, quantile_levels=list(QUANTILES))
            for offset, forecast in enumerate(forecasts):
                position = positions[batch_start + offset]
                values = forecast[0, :, -1].detach().cpu().numpy()
                current = price[position]
                for name, value in zip(columns, values):
                    result.loc[position, name] = (float(value) - current) * 10_000
                result.loc[position, "chronos_available"] = True
        return result, {
            "selected_backbone": config.model_id,
            "device": device,
            "inference_seconds": time.perf_counter() - started,
            "covariate_count": len(covariate_columns),
        }
    except Exception as exc:
        for column in ("chronos_return_q10_bps", "chronos_return_q50_bps", "chronos_return_q90_bps"):
            if column in result:
                result[column] = np.nan
        result["chronos_available"] = False
        return result, {
            "selected_backbone": "chronos_unavailable",
            "fallback_candidate": "price_only",
            "fallback_reason": f"{type(exc).__name__}: {exc}",
        }


def _fit_neural(
    panel: pd.DataFrame,
    train_indices: np.ndarray,
    labels: np.ndarray,
    returns_bps: np.ndarray,
    config: ResearchConfig,
    calibration_indices: np.ndarray,
    price_columns: list[str],
    modality_columns: dict[str, list[str]],
    mask_columns: dict[str, list[str]],
    output_path: Path,
    seed_offset: int,
) -> dict[str, Any]:
    seed = config.seed + seed_offset
    torch.manual_seed(seed)
    np.random.seed(seed)
    price, price_transform = _standardize(_matrix(panel, price_columns), train_indices)
    sequences = _sequences(price, config.sequence_length)
    modality_arrays: dict[str, np.ndarray] = {}
    modality_transforms: dict[str, dict[str, list[float]]] = {}
    modality_masks: dict[str, np.ndarray] = {}
    for name, columns in modality_columns.items():
        modality_arrays[name], modality_transforms[name] = _standardize(_matrix(panel, columns), train_indices)
        modality_masks[name] = _mask_matrix(panel, mask_columns.get(name, []))

    model = MultimodalForecaster(
        price_features=sequences.shape[-1],
        modality_dims={name: values.shape[-1] for name, values in modality_arrays.items()},
        quantiles=QUANTILES,
    )
    optimizer = torch.optim.AdamW(model.parameters(), lr=0.001, weight_decay=0.01)
    q = torch.tensor(QUANTILES, dtype=torch.float32)
    train_indices = train_indices.astype(np.int64)
    model.train()
    for _ in range(max(1, config.epochs)):
        order = torch.randperm(len(train_indices))
        for offset in range(0, len(order), max(1, config.batch_size)):
            selected = train_indices[order[offset : offset + config.batch_size].numpy()]
            batch = {
                "price_seq": torch.tensor(sequences[selected], dtype=torch.float32),
                "modalities": {
                    name: torch.tensor(values[selected], dtype=torch.float32) for name, values in modality_arrays.items()
                },
                "masks": {name: torch.tensor(values[selected], dtype=torch.bool) for name, values in modality_masks.items()},
            }
            y_class = torch.tensor(labels[selected], dtype=torch.long)
            y_return = torch.tensor(returns_bps[selected], dtype=torch.float32)
            output = model(batch)
            class_loss = torch.nn.functional.cross_entropy(output["direction_logits"], y_class)
            error = y_return.unsqueeze(-1) - output["return_quantiles"]
            pinball = torch.maximum(q * error, (q - 1.0) * error).mean()
            sigma = output["uncertainty"].clamp_min(0.1)
            median = output["return_quantiles"][:, 1]
            nll = (0.5 * ((y_return - median) / sigma).square() + torch.log(sigma)).mean()
            loss = class_loss + 0.1 * pinball + 0.01 * nll
            optimizer.zero_grad(set_to_none=True)
            loss.backward()
            torch.nn.utils.clip_grad_norm_(model.parameters(), 1.0)
            optimizer.step()
    model.eval()
    calibration_logits = []
    with torch.inference_mode():
        for offset in range(0, len(calibration_indices), max(1, config.batch_size)):
            selected = calibration_indices[offset : offset + config.batch_size]
            batch = {
                "price_seq": torch.tensor(sequences[selected], dtype=torch.float32),
                "modalities": {
                    name: torch.tensor(values[selected], dtype=torch.float32) for name, values in modality_arrays.items()
                },
                "masks": {name: torch.tensor(values[selected], dtype=torch.bool) for name, values in modality_masks.items()},
            }
            calibration_logits.append(model(batch)["direction_logits"].cpu().numpy())
    calibration_temperature = (
        _temperature(np.concatenate(calibration_logits), labels[calibration_indices]) if calibration_logits else 1.0
    )
    torch.save(
        {
            "state_dict": model.state_dict(),
            "price_columns": price_columns,
            "modality_columns": modality_columns,
            "mask_columns": mask_columns,
            "price_transform": price_transform,
            "modality_transforms": modality_transforms,
            "sequence_length": config.sequence_length,
            "quantiles": QUANTILES,
            "return_scale": 10_000,
            "calibration_temperature": calibration_temperature,
        },
        output_path,
    )
    return {
        "price_features": len(price_columns),
        "modality_dimensions": {name: len(columns) for name, columns in modality_columns.items()},
        "training_rows": len(train_indices),
        "calibration_rows": len(calibration_indices),
        "calibration_temperature": calibration_temperature,
        "seed": seed,
    }


def fit_candidates(panel: pd.DataFrame, config: ResearchConfig) -> dict[str, Path]:
    if panel.empty or TARGET not in panel:
        raise ValueError("Training panel is empty or has no one-hour return label")
    panel = panel.sort_values("decision_at").reset_index(drop=True)
    labeled = panel[TARGET].notna().to_numpy()
    valid_indices = np.flatnonzero(labeled)
    if len(valid_indices) < 80:
        raise ValueError(f"Only {len(valid_indices)} labeled rows; at least 80 are required for exploratory training")
    cutoff = int(len(valid_indices) * 0.8)
    development_indices = valid_indices[:cutoff]
    development_pool = development_indices[:-4]
    calibration_start = int(len(development_pool) * 0.8)
    train_indices = development_pool[: max(0, calibration_start - 4)]
    calibration_indices = development_pool[calibration_start:]
    if len(train_indices) < 30 or len(calibration_indices) < 8:
        raise ValueError("Too few rows remain for training/calibration after final reservation and four-candle purges")

    panel, backbone = _chronos_features(panel, config)
    target = pd.to_numeric(panel[TARGET], errors="coerce").to_numpy(dtype=float)
    classes = np.where(target > config.neutral_return_threshold, 2, np.where(target < -config.neutral_return_threshold, 0, 1))
    returns_bps = target * 10_000
    numeric_columns = [
        column
        for column in panel.select_dtypes(include=[np.number, "bool"]).columns
        if column != TARGET and not column.startswith("target_")
    ]
    price_columns, groups, masks = _columns(panel)
    if not price_columns:
        raise ValueError("Panel has no XAUUSD price features")

    run_id = uuid.uuid4().hex[:12]
    run_dir = Path(config.artifact_dir) / run_id
    run_dir.mkdir(parents=True, exist_ok=True)
    artifacts: dict[str, Path] = {}
    no_change = run_dir / "no_change.json"
    no_change.write_text(json.dumps({"kind": "no_change", "direction": "flat", "expected_return": 0.0}, indent=2) + "\n")
    artifacts["no_change"] = no_change

    tree_values, tree_transform = _standardize(_matrix(panel, numeric_columns), train_indices)
    x_train = tree_values[train_indices]
    y_train = classes[train_indices]
    classifier: Any = (
        HistGradientBoostingClassifier(max_iter=100, max_leaf_nodes=15, l2_regularization=1.0, random_state=config.seed)
        if len(np.unique(y_train)) > 1
        else DummyClassifier(strategy="prior")
    )
    regressor = HistGradientBoostingRegressor(max_iter=100, max_leaf_nodes=15, l2_regularization=1.0, random_state=config.seed)
    classifier.fit(x_train, y_train)
    regressor.fit(x_train, returns_bps[train_indices])
    calibration_matrix = tree_values[calibration_indices]
    raw_calibration_probabilities = classifier.predict_proba(calibration_matrix)
    classes_present = np.asarray(classifier.classes_, dtype=int)
    aligned_calibration_probabilities = np.full((len(calibration_indices), 3), 1e-8)
    aligned_calibration_probabilities[:, classes_present] = raw_calibration_probabilities
    aligned_calibration_probabilities /= aligned_calibration_probabilities.sum(axis=1, keepdims=True)
    tree_temperature = _temperature(np.log(aligned_calibration_probabilities), classes[calibration_indices])
    residual_quantiles = np.quantile(
        returns_bps[calibration_indices] - regressor.predict(calibration_matrix), QUANTILES
    ).tolist()
    import joblib

    tabular_path = run_dir / "tabular.joblib"
    joblib.dump(
        {
            "classifier": classifier,
            "regressor": regressor,
            "columns": numeric_columns,
            "transform": tree_transform,
            "calibration_temperature": tree_temperature,
            "residual_quantiles": residual_quantiles,
        },
        tabular_path,
    )
    artifacts["tabular"] = tabular_path

    price_only_path = run_dir / "price_only.pt"
    _fit_neural(
        panel,
        train_indices,
        classes,
        returns_bps,
        config,
        calibration_indices,
        price_columns,
        {},
        {},
        price_only_path,
        1,
    )
    artifacts["price_only"] = price_only_path

    modality_path = run_dir / "multimodal.pt"
    multimodal_metadata = _fit_neural(
        panel,
        train_indices,
        classes,
        returns_bps,
        config,
        calibration_indices,
        price_columns,
        groups,
        masks,
        modality_path,
        2,
    )
    artifacts["multimodal"] = modality_path

    dependencies = {}
    for name in ("numpy", "pandas", "torch", "transformers", "chronos-forecasting", "scikit-learn"):
        try:
            dependencies[name] = metadata.version(name)
        except metadata.PackageNotFoundError:
            dependencies[name] = "not-installed"
    manifest = {
        "run_id": run_id,
        "data_version": hashlib.sha256(
            pd.util.hash_pandas_object(panel.drop(columns=[TARGET]), index=True).values.tobytes()
        ).hexdigest(),
        "feature_version": "xauusd-asof-v1",
        "model_versions": {"finbert": config.finbert_model_id, **backbone},
        "data_window": {
            "panel_start": str(panel["decision_at"].min()),
            "panel_end": str(panel["decision_at"].max()),
            "training_start": str(panel.iloc[train_indices[0]]["decision_at"]),
            "training_end": str(panel.iloc[train_indices[-1]]["decision_at"]),
            "calibration_start": str(panel.iloc[calibration_indices[0]]["decision_at"]),
            "untouched_final_start": str(panel.iloc[valid_indices[cutoff]]["decision_at"]),
        },
        "split": {
            "train_rows": len(train_indices),
            "calibration_rows": len(calibration_indices),
            "purge_candles": 4,
            "final_boundary_purge_candles": 4,
            "final_fraction": 0.2,
            "calibration_fraction_of_development": 0.2,
        },
        "seed": config.seed,
        "config": config.to_dict(),
        "cost_assumptions": {
            "spread_bps": config.spread_bps,
            "commission_per_side_bps": config.commission_per_side_bps,
            "slippage_per_side_bps": config.slippage_per_side_bps,
            "cost_sensitivity_bps": config.cost_sensitivity_bps,
        },
        "return_scale": "basis_points (model); divide by 10000 for log return",
        "direction_classes": {"0": "down", "1": "flat", "2": "up"},
        "neutral_return_threshold": config.neutral_return_threshold,
        "features": {"tabular": numeric_columns, "price_sequence": price_columns, "modalities": groups},
        "model_details": multimodal_metadata,
        "coverage": coverage_report(panel),
        "dependencies": dependencies,
        "artifacts": {name: path.name for name, path in artifacts.items()},
        "python": platform.python_version(),
    }
    (run_dir / "run.json").write_text(json.dumps(manifest, indent=2, default=str) + "\n")
    artifacts["manifest"] = run_dir / "run.json"
    return artifacts
