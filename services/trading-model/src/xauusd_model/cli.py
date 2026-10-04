from __future__ import annotations

import argparse
import json
import os
import platform
import time
from pathlib import Path

from .config import ResearchConfig


def _inspect_backbone(config: ResearchConfig, run_inference: bool) -> dict:
    import psutil
    import torch

    official_model = config.model_id == "amazon/chronos-2"
    result = {
        "model_id": config.model_id,
        "license": "Apache-2.0" if official_model else "verify model-specific license",
        "model_parameters": 120_000_000 if official_model else None,
        "supports_multivariate_and_covariate_forecasting": True if official_model else None,
        "metadata_source": "https://github.com/amazon-science/chronos-forecasting",
        "model_card": f"https://huggingface.co/{config.model_id}",
        "python": platform.python_version(),
        "cpu_memory_available_bytes": psutil.virtual_memory().available,
        "cuda_available": torch.cuda.is_available(),
        "selected_backbone": config.model_id,
        "inference_seconds": None,
        "inference_status": "not run; pass --run-inference to load model weights",
    }
    if torch.cuda.is_available():
        result["cuda_memory_free_bytes"] = torch.cuda.mem_get_info()[0]
    if not run_inference:
        return result

    try:
        from chronos import Chronos2Pipeline

        device = "cuda" if torch.cuda.is_available() else "cpu"
        started = time.perf_counter()
        pipeline = Chronos2Pipeline.from_pretrained(config.model_id, device_map=device)
        levels = [0.1, 0.5, 0.9]
        forecasts, _ = pipeline.predict_quantiles(
            [
                {
                    "target": torch.linspace(1.0, 2.0, 32),
                    "past_covariates": {"market_return": torch.zeros(32)},
                }
            ],
            prediction_length=4,
            quantile_levels=levels,
        )
        forecast = forecasts[0]
        if tuple(forecast.shape) != (1, 4, len(levels)):
            raise RuntimeError(f"unexpected covariate forecast shape: {tuple(forecast.shape)}")
        result["inference_seconds"] = time.perf_counter() - started
        result["inference_status"] = "ok"
        result["device"] = device
        result["forecast_shape"] = list(forecast.shape)
    except Exception as exc:
        result["selected_backbone"] = "price_only"
        result["inference_status"] = "fallback"
        result["fallback_reason"] = f"{type(exc).__name__}: {exc}"
    return result


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="xauusd-model")
    subparsers = parser.add_subparsers(dest="command", required=True)

    inspect = subparsers.add_parser("inspect-backbone", help="inspect Chronos-2 suitability")
    inspect.add_argument("--config")
    inspect.add_argument("--run-inference", action="store_true", help="download/load weights and time a small CPU/GPU forecast")

    coverage = subparsers.add_parser("report-coverage", help="report point-in-time source coverage")
    coverage.add_argument("--config")
    coverage.add_argument("--from", dest="from_date", required=True)
    coverage.add_argument("--to", dest="to_date", required=True)
    coverage.add_argument("--output")

    train = subparsers.add_parser("train", help="fit benchmark and multimodal research candidates")
    train.add_argument("--config", required=True)
    train.add_argument("--from", dest="from_date")
    train.add_argument("--to", dest="to_date")

    evaluate = subparsers.add_parser("evaluate", help="run walk-forward and untouched-final-interval evaluation")
    evaluate.add_argument("--config", required=True)
    evaluate.add_argument("--run", required=True)

    predict = subparsers.add_parser("predict", help="create an offline paper prediction for an as-of timestamp")
    predict.add_argument("--config", required=True)
    predict.add_argument("--run", required=True)
    predict.add_argument("--as-of", required=True)
    predict.add_argument("--output")

    return parser


def _build_panel(config: ResearchConfig, start: str, end: str, include_text: bool = True):
    import pandas as pd

    from .features import build_asof_panel
    from .store import load_market_panel, load_side_channels

    market = load_market_panel(config, start, end)
    side_channels = load_side_channels(config, start, end)
    panel = build_asof_panel(market, side_channels, decision_start=start, decision_end=end)
    if include_text:
        from .text import attach_news_embeddings, encode_news

        embeddings = encode_news(
            side_channels.get("news", pd.DataFrame()),
            checkpoint=config.finbert_model_id,
            batch_size=max(1, config.batch_size // 4),
            cache_dir=Path(config.artifact_dir) / "cache",
        )
        panel = attach_news_embeddings(panel, embeddings)
    return panel


def main() -> None:
    args = _parser().parse_args()
    config = ResearchConfig.load(args.config)
    if args.command == "inspect-backbone":
        print(json.dumps(_inspect_backbone(config, args.run_inference), indent=2))
        return
    if args.command == "report-coverage":
        from .features import build_asof_panel, coverage_report
        from .store import load_market_panel, load_side_channels

        start, end = args.from_date, args.to_date
        market = load_market_panel(config, start, end)
        channels = load_side_channels(config, start, end)
        panel = build_asof_panel(market, channels, decision_start=start, decision_end=end)
        report = coverage_report(panel)
        report["requested_window"] = {"from": start, "to": end}
        report["config"] = config.to_dict()
        rendered = json.dumps(report, indent=2, default=str)
        if args.output:
            output = Path(args.output)
            output.parent.mkdir(parents=True, exist_ok=True)
            if output.suffix.lower() == ".md":
                lines = [
                    "# XAUUSD source coverage",
                    "",
                    f"Window: `{start}` to `{end}`",
                    f"Decision rows: {report.get('rows', 0)}; labeled rows: {report.get('target_rows', 0)}",
                    "",
                    "| Source | Coverage | Rows | Median age (s) | 95th percentile age (s) |",
                    "| --- | ---: | ---: | ---: | ---: |",
                ]
                for name, source in report.get("sources", {}).items():
                    lines.append(
                        f"| {name} | {source['available_fraction']:.1%} | {source['available_rows']} | "
                        f"{source['age_seconds_p50'] if source['age_seconds_p50'] is not None else '—'} | "
                        f"{source['age_seconds_p95'] if source['age_seconds_p95'] is not None else '—'} |"
                    )
                output.write_text("\n".join(lines) + os.linesep)
            else:
                output.write_text(rendered + os.linesep)
        print(rendered)
        return
    if args.command == "train":
        from dataclasses import replace
        from .train import fit_candidates

        start = args.from_date or config.from_date
        end = args.to_date or config.to_date
        if not start or not end:
            raise SystemExit("training requires --from/--to or from_date/to_date in the JSON config")
        config = replace(config, from_date=start, to_date=end)
        panel = _build_panel(config, start, end)
        artifacts = fit_candidates(panel, config)
        print(json.dumps({name: str(path) for name, path in artifacts.items()}, indent=2))
        return
    if args.command == "evaluate":
        from dataclasses import replace

        from .artifacts import write_evaluation, write_model_card
        from .evaluate import evaluate_run, walk_forward

        run_dir = Path(args.run)
        manifest = json.loads((run_dir / "run.json").read_text())
        window = manifest["data_window"]
        start, end = window["panel_start"], window["panel_end"]
        config = replace(config, from_date=start, to_date=end)
        panel = _build_panel(config, start, end)
        research = walk_forward(panel, config)
        final = evaluate_run(panel, run_dir, config)
        prediction_rows = final.pop("predictions")
        for name, predictions in prediction_rows.items():
            predictions.to_csv(run_dir / f"predictions-final-{name}.csv", index=False)
        report = {"walk_forward": research, "final_holdout": final}
        write_evaluation(run_dir, report)
        write_model_card(run_dir, report)
        print(json.dumps(report, indent=2, default=str))
        return
    if args.command == "predict":
        from dataclasses import replace

        from .evaluate import predict_asof

        run_dir = Path(args.run)
        manifest = json.loads((run_dir / "run.json").read_text())
        start = manifest["data_window"]["panel_start"]
        config = replace(config, from_date=start, to_date=args.as_of)
        panel = _build_panel(config, start, args.as_of)
        prediction = predict_asof(panel, run_dir, args.as_of)
        rendered = json.dumps(prediction, indent=2)
        if args.output:
            output = Path(args.output)
            output.parent.mkdir(parents=True, exist_ok=True)
            output.write_text(rendered + os.linesep)
        print(rendered)


if __name__ == "__main__":
    main()
