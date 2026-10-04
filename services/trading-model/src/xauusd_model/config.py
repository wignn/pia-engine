from __future__ import annotations

import json
import os
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any


@dataclass(frozen=True)
class ResearchConfig:
    postgres_url: str | None = None
    clickhouse_url: str | None = None
    artifact_dir: str = "artifacts/xauusd-research"
    model_id: str = "amazon/chronos-2"
    finbert_model_id: str = "yiyanghkust/finbert-tone"
    seed: int = 42
    from_date: str | None = None
    to_date: str | None = None

    @classmethod
    def load(cls, path: str | None = None) -> ResearchConfig:
        values: dict[str, Any] = {}
        if path:
            values = json.loads(Path(path).read_text())
        values.setdefault("postgres_url", os.getenv("TRADING_MODEL_POSTGRES_URL"))
        values.setdefault("clickhouse_url", os.getenv("TRADING_MODEL_CLICKHOUSE_URL"))
        values.setdefault("artifact_dir", os.getenv("TRADING_MODEL_ARTIFACT_DIR", cls.artifact_dir))
        values.setdefault("finbert_model_id", os.getenv("SENTIMENT_MODEL", cls.finbert_model_id))
        known = cls.__dataclass_fields__.keys()
        return cls(**{key: value for key, value in values.items() if key in known})

    def to_dict(self) -> dict[str, Any]:
        result = asdict(self)
        # URLs may contain credentials and must never be copied into run manifests.
        for key in ("postgres_url", "clickhouse_url"):
            if result[key]:
                result[key] = "configured"
        return result
