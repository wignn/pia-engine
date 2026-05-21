import os
import re
import logging
from functools import lru_cache
from typing import Any, Dict, List, Tuple

logger = logging.getLogger("finbert-service.analyzer")


class AdvancedSentimentAnalyzer:
    def __init__(self):
        self.pipeline = None
        self.model_name = os.getenv("SENTIMENT_MODEL", "yiyanghkust/finbert-tone")
        self.max_length = int(os.getenv("MAX_TOKENS", "512"))
        self.neutral_threshold = float(os.getenv("NEUTRAL_THRESHOLD", "0.55"))
        self.margin_threshold = float(os.getenv("MARGIN_THRESHOLD", "0.10"))

    def initialize(self):
        if self.pipeline is not None:
            return

        logger.info("Loading HF pipeline with '%s'...", self.model_name)
        from transformers import AutoModelForSequenceClassification, AutoTokenizer, pipeline

        tokenizer = AutoTokenizer.from_pretrained(self.model_name, use_fast=False)
        model = AutoModelForSequenceClassification.from_pretrained(self.model_name)

        self.pipeline = pipeline(
            "sentiment-analysis",
            model=model,
            tokenizer=tokenizer,
            device=-1,  # CPU only, safe for VPS
            top_k=None,
        )
        logger.info("HF model and pipeline initialized successfully.")

    def _normalize_label(self, label: str) -> str:
        label = label.lower().strip()
        mapping = {
            "label_0": "negative",
            "label_1": "neutral",
            "label_2": "positive",
            "negative": "negative",
            "neutral": "neutral",
            "positive": "positive",
            "neg": "negative",
            "neu": "neutral",
            "pos": "positive",
        }
        return mapping.get(label, label)

    def _compact_text(self, text: str, title: str | None = None) -> str:
        text = re.sub(r"\s+", " ", text or "").strip()
        title = re.sub(r"\s+", " ", title or "").strip()

        sentences = re.split(r"(?<=[.!?])\s+", text)
        selected = []
        if title:
            selected.append(title)
        selected.extend(sentences[:10])

        compact = " ".join(s for s in selected if s).strip()
        return compact or text[:2500]

    @lru_cache(maxsize=2048)
    def _cached_inference(self, text: str) -> List[Dict[str, Any]]:
        self.initialize()
        result = self.pipeline(
            text,
            truncation=True,
            max_length=self.max_length,
        )
        return result[0] if result and isinstance(result[0], list) else result

    def _distribution(self, predictions: List[Dict[str, Any]]) -> Dict[str, float]:
        dist = {"positive": 0.0, "negative": 0.0, "neutral": 0.0}
        for pred in predictions:
            label = self._normalize_label(str(pred.get("label", "")))
            if label in dist:
                dist[label] = float(pred.get("score", 0.0))
        return dist

    def _calibrate(self, dist: Dict[str, float]) -> Tuple[str, float]:
        ordered = sorted(dist.items(), key=lambda x: x[1], reverse=True)
        top_label, top_score = ordered[0]
        second_score = ordered[1][1] if len(ordered) > 1 else 0.0
        margin = top_score - second_score

        if top_score < self.neutral_threshold or margin < self.margin_threshold:
            return "mixed", top_score
        return top_label, top_score

    def analyze(self, text: str, title: str | None = None, language: str | None = None) -> Dict[str, Any]:
        if not text or text.strip() == "":
            return {
                "sentiment": "neutral",
                "score": 1.0,
                "distribution": {"positive": 0.0, "negative": 0.0, "neutral": 1.0},
                "highlights": [],
                "entities": {"tickers": [], "currencies": []},
                "model_used": self.model_name,
            }

        self.initialize()

        try:
            model_text = self._compact_text(text, title)
            raw_predictions = self._cached_inference(model_text)
            dist = self._distribution(raw_predictions)
            top_sentiment, top_score = self._calibrate(dist)

            highlights = self._extract_highlights(text)
            entities = self._extract_entities(text)

            return {
                "sentiment": top_sentiment,
                "score": top_score,
                "distribution": dist,
                "highlights": highlights,
                "entities": entities,
                "model_used": self.model_name,
            }

        except Exception as e:
            logger.error("Error during advanced sentiment analysis: %s", e, exc_info=True)
            return {
                "sentiment": "neutral",
                "score": 0.0,
                "distribution": {"positive": 0.0, "negative": 0.0, "neutral": 1.0},
                "highlights": [],
                "entities": {"tickers": [], "currencies": []},
                "model_used": self.model_name,
            }

    def _extract_highlights(self, text: str) -> List[Dict[str, Any]]:
        sentences = re.split(r"(?<!\w\.\w.)(?<![A-Z][a-z]\.)(?<=\.|\?)\s", text)
        highlights = []

        for s in sentences:
            s = s.strip()
            if len(s) < 25 or len(s) > 300:
                continue

            try:
                s_preds = self._cached_inference(s)
                s_dist = self._distribution(s_preds)
                s_top, s_score = self._calibrate(s_dist)

                if s_top in ("positive", "negative") and s_score > 0.78:
                    highlights.append({
                        "sentence": s,
                        "sentiment": s_top,
                        "score": s_score,
                    })
            except Exception:
                continue

        highlights.sort(key=lambda x: x["score"], reverse=True)
        return highlights[:3]

    def _extract_entities(self, text: str) -> Dict[str, List[str]]:
        tickers = set()
        currencies = set()

        stock_matches = re.findall(r"\$[A-Z]{1,5}\b", text)
        for m in stock_matches:
            tickers.add(m.upper())

        forex_slashes = re.findall(r"\b[A-Z]{3}/[A-Z]{3}\b", text)
        for m in forex_slashes:
            currencies.add(m.upper())

        common_currencies = r"\b(EUR|USD|GBP|JPY|AUD|CAD|CHF|NZD)(USD|JPY|GBP|EUR|CAD|CHF|AUD|NZD)\b"
        forex_merges = re.findall(common_currencies, text)
        for m1, m2 in forex_merges:
            if m1 != m2:
                currencies.add(f"{m1.upper()}/{m2.upper()}")

        crypto_keywords = r"\b(BTC|ETH|SOL|ADA|XRP|DOT|DOGE|LTC|LINK|SHIB|AVAX|MATIC|BNB)\b"
        crypto_matches = re.findall(crypto_keywords, text)
        for m in crypto_matches:
            tickers.add(m.upper())

        index_keywords = r"\b(SPX|SP500|S&P 500|NASDAQ|NDX|DOW|DJIA|DXY|XAU|XAUUSD|GOLD)\b"
        index_matches = re.findall(index_keywords, text, flags=re.IGNORECASE)
        for m in index_matches:
            tickers.add(m.upper())

        return {
            "tickers": sorted(tickers),
            "currencies": sorted(currencies),
        }
