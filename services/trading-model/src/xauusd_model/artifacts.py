from __future__ import annotations

import hashlib
import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any


@dataclass(frozen=True)
class ArtifactRef:
    name: str
    path: Path
    sha256: str


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def collect_artifacts(run_dir: Path) -> list[ArtifactRef]:
    refs = []
    for path in sorted(run_dir.iterdir()):
        if path.is_file() and path.name not in {"run.json", "evaluation.json", "model-card.md"}:
            refs.append(ArtifactRef(path.stem, path, sha256_file(path)))
    return refs


def write_evaluation(run_dir: Path, report: dict[str, Any]) -> Path:
    path = run_dir / "evaluation.json"
    path.write_text(json.dumps(report, indent=2, default=str) + "\n")
    return path


def write_model_card(run_dir: Path, report: dict[str, Any]) -> Path:
    manifest_path = run_dir / "run.json"
    manifest = json.loads(manifest_path.read_text()) if manifest_path.exists() else {}
    hashes = collect_artifacts(run_dir)
    manifest["artifact_hashes"] = {ref.path.name: ref.sha256 for ref in hashes}
    manifest_path.write_text(json.dumps(manifest, indent=2, default=str) + "\n")
    window = manifest.get("data_window", {})
    final = report.get("final_holdout", {})
    metrics = final.get("models", {})
    lines = [
        "# XAUUSD research model card",
        "",
        f"Run: `{manifest.get('run_id', run_dir.name)}`",
        f"Data window: `{window.get('panel_start', 'unknown')}` to `{window.get('panel_end', 'unknown')}`",
        f"Feature version: `{manifest.get('feature_version', 'unknown')}`",
        f"Backbone: `{manifest.get('model_versions', {}).get('selected_backbone', 'not recorded')}`",
        "",
        "## Intended use",
        "",
        "Offline research and paper evaluation of one-hour XAUUSD return distributions from 15-minute decision points. This artifact does not send orders and makes no profitability claim.",
        "",
        "## Known data limits",
        "",
        "- The available XAUUSD history is short and results are exploratory.",
        "- GLD options are an equity ETF proxy, not XAUUSD options.",
        "- Append-only options and macro history begins only after migration 029 and its service deployment; overwritten historical values are not reconstructed.",
        "- COT release timestamps are absent; the pipeline uses conservative database update availability.",
        "- Check source licenses before redistributing article-derived embeddings, feature data, or model artifacts.",
        "",
        "## Final interval metrics",
        "",
        "| Model | Rows | Brier | Log loss | Return MAE |",
        "| --- | ---: | ---: | ---: | ---: |",
    ]
    for name, values in metrics.items():
        lines.append(
            f"| {name} | {values.get('rows', 0)} | {values.get('brier_score', float('nan')):.4f} | "
            f"{values.get('log_loss', float('nan')):.4f} | {values.get('return_mae_bps', float('nan')):.3f} bps |"
        )
    lines += ["", "## Artifact hashes", ""]
    lines.extend(f"- `{ref.path.name}`: SHA-256 `{ref.sha256}`" for ref in hashes)
    lines += ["", "Metrics and calibration are research diagnostics, not evidence of deployable alpha.", ""]
    path = run_dir / "model-card.md"
    path.write_text("\n".join(lines))
    return path
