from __future__ import annotations

import hashlib
import re
from datetime import timedelta
from pathlib import Path

import numpy as np
import pandas as pd


def _cache_key(row: pd.Series, checkpoint: str) -> str:
    source_version = "|".join(str(row.get(column, "")) for column in ("title", "summary", "content"))
    identity = "|".join(
        (str(row.get("id", "")), str(row.get("processed_at", "")), str(row.get("available_at", "")), checkpoint, source_version)
    )
    return hashlib.sha256(identity.encode()).hexdigest()


def encode_news(
    rows: pd.DataFrame,
    checkpoint: str,
    batch_size: int = 16,
    cache_dir: str | Path | None = None,
) -> pd.DataFrame:
    embedding_names = [f"embedding_{index:04d}" for index in range(768)]
    if rows.empty:
        return pd.DataFrame(
            columns=["id", "available_at", "processed_at", "sentiment", "impact_level", "currency_pairs", "cache_key", *embedding_names]
        )
    data = rows.copy().reset_index(drop=True)
    data["id"] = data["id"].astype(str)
    data["available_at"] = pd.to_datetime(data["available_at"], utc=True, errors="coerce")
    if "processed_at" not in data:
        data["processed_at"] = data["available_at"]
    metadata_columns = [column for column in ("sentiment", "impact_level", "currency_pairs") if column in data]
    data["cache_key"] = data.apply(lambda row: _cache_key(row, checkpoint), axis=1)

    cache_path = None
    cached = pd.DataFrame()
    if cache_dir:
        checkpoint_key = hashlib.sha256(checkpoint.encode()).hexdigest()[:12]
        cache_path = Path(cache_dir) / f"finbert-{checkpoint_key}.parquet"
        if cache_path.exists():
            cached = pd.read_parquet(cache_path)
    cached_keys = set(cached.get("cache_key", pd.Series(dtype=str)).astype(str))
    missing = data[~data["cache_key"].isin(cached_keys)].copy()
    encoded_parts: list[pd.DataFrame] = []
    if not missing.empty:
        import torch
        from transformers import AutoModel, AutoTokenizer

        tokenizer = AutoTokenizer.from_pretrained(checkpoint)
        model = AutoModel.from_pretrained(checkpoint)
        device = "cuda" if torch.cuda.is_available() else "cpu"
        model.to(device).eval()
        titles = missing["title"].fillna("") if "title" in missing else pd.Series("", index=missing.index)
        summaries = missing["summary"].fillna("") if "summary" in missing else pd.Series("", index=missing.index)
        contents = missing["content"].fillna("") if "content" in missing else pd.Series("", index=missing.index)
        texts = [
            re.sub(r"\s+", " ", f"{title}. {summary}. {content[:8000]}").strip()[:12000]
            for title, summary, content in zip(titles, summaries, contents)
        ]
        vectors: list[np.ndarray] = []
        with torch.inference_mode():
            for start in range(0, len(texts), max(1, batch_size)):
                batch = tokenizer(
                    texts[start : start + batch_size],
                    padding=True,
                    truncation=True,
                    max_length=512,
                    return_tensors="pt",
                ).to(device)
                hidden = model(**batch).last_hidden_state
                mask = batch["attention_mask"].unsqueeze(-1).to(hidden.dtype)
                pooled = (hidden * mask).sum(dim=1) / mask.sum(dim=1).clamp_min(1)
                vectors.extend(pooled.cpu().numpy().astype(np.float32))
        encoded = pd.DataFrame(np.stack(vectors), columns=embedding_names, index=missing.index)
        encoded_parts.append(pd.concat([missing[["id", "available_at", "processed_at", *metadata_columns, "cache_key"]], encoded], axis=1))

    fresh = pd.concat(encoded_parts, ignore_index=True) if encoded_parts else pd.DataFrame()
    result = pd.concat([cached, fresh], ignore_index=True) if not cached.empty else fresh
    result = result.drop_duplicates("cache_key", keep="last")
    if cache_path and not fresh.empty:
        cache_path.parent.mkdir(parents=True, exist_ok=True)
        result.to_parquet(cache_path, index=False)
    wanted = set(data["cache_key"])
    return result[result["cache_key"].isin(wanted)].reset_index(drop=True)


def attach_news_embeddings(
    panel: pd.DataFrame,
    embeddings: pd.DataFrame,
    freshness: timedelta = timedelta(hours=8),
) -> pd.DataFrame:
    result = panel.copy()
    if embeddings.empty:
        result["news_embedding_available"] = False
        result["news_embedding_age_seconds"] = np.nan
        result["news_embedding_count"] = 0
        return result
    columns = [column for column in embeddings.columns if column.startswith("embedding_")]
    events = embeddings.copy()
    events["available_at"] = pd.to_datetime(events["available_at"], utc=True, errors="coerce")
    events = events.dropna(subset=["available_at"]).sort_values("available_at").reset_index(drop=True)
    timestamps = events["available_at"].astype("int64").to_numpy()
    vectors = events[columns].to_numpy(dtype=np.float32)
    window_ns = int(freshness.total_seconds() * 1_000_000_000)
    decisions = pd.to_datetime(result["decision_at"], utc=True)
    aggregate = np.full((len(result), len(columns)), np.nan, dtype=np.float32)
    ages = np.full(len(result), np.nan)
    counts = np.zeros(len(result), dtype=np.int32)
    for index, decision in enumerate(decisions):
        right = int(np.searchsorted(timestamps, decision.value, side="right"))
        left = int(np.searchsorted(timestamps, decision.value - window_ns, side="left"))
        if right <= left:
            continue
        aggregate[index] = vectors[left:right].mean(axis=0)
        ages[index] = (decision.value - timestamps[right - 1]) / 1_000_000_000
        counts[index] = right - left
    for column_index, column in enumerate(columns):
        result[f"news_{column}"] = aggregate[:, column_index]
    result["news_embedding_available"] = counts > 0
    result["news_embedding_age_seconds"] = ages
    result["news_embedding_count"] = counts
    return result
