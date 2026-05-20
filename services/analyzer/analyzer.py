import re
import logging
from functools import lru_cache
from typing import Dict, Any, List

logger = logging.getLogger("finbert-service.analyzer")

class AdvancedSentimentAnalyzer:
    def __init__(self):
        self.pipeline = None
        self.model_name = "yiyanghkust/finbert-tone"

    def initialize(self):
        if self.pipeline is not None:
            return
        
        logger.info(f"Loading HF pipeline with '{self.model_name}'...")
        from transformers import BertTokenizer, AutoModelForSequenceClassification, pipeline
        
        tokenizer = BertTokenizer.from_pretrained(self.model_name)
        model = AutoModelForSequenceClassification.from_pretrained(self.model_name)
        
        # top_k=None ensures we get scores for all labels (positive, negative, neutral)
        self.pipeline = pipeline(
            "sentiment-analysis",
            model=model,
            tokenizer=tokenizer,
            device=-1, # CPU by default
            top_k=None
        )
        logger.info("HF model and pipeline initialized successfully.")

    @lru_cache(maxsize=2048)
    def _cached_inference(self, text: str) -> List[Dict[str, Any]]:
        """
        In-memory cached execution of the sentiment pipeline.
        Cuts CPU overhead for duplicate ticker feeds/RSS entries.
        """
        # Truncate text to BERT context length limit
        truncated = text[:1500]
        return self.pipeline(truncated)[0]

    def analyze(self, text: str) -> Dict[str, Any]:
        if not text or text.strip() == "":
            return {
                "sentiment": "neutral",
                "score": 1.0,
                "distribution": {"positive": 0.0, "negative": 0.0, "neutral": 1.0},
                "highlights": [],
                "entities": {"tickers": [], "currencies": []}
            }
        
        self.initialize()
        
        try:
            # 1. Run Sentiment Model Inference
            raw_predictions = self._cached_inference(text)
            
            # Map predictions to label dictionary
            dist = {}
            for pred in raw_predictions:
                label = pred["label"].lower()
                dist[label] = float(pred["score"])
                
            # Determine overall sentiment (highest score)
            top_sentiment = max(dist, key=dist.get)
            top_score = dist[top_sentiment]
            
            # 2. Extract Key Sentences / Highlights
            sentences = re.split(r'(?<!\w\.\w.)(?<![A-Z][a-z]\.)(?<=\.|\?)\s', text)
            highlights = []
            for s in sentences:
                s = s.strip()
                if len(s) < 25 or len(s) > 300: # filter out boilerplate or giant paragraphs
                    continue
                
                # Get sentiment of this specific sentence
                s_preds = self._cached_inference(s)
                s_dist = {p["label"].lower(): float(p["score"]) for p in s_preds}
                s_top = max(s_dist, key=s_dist.get)
                
                if s_top in ("positive", "negative") and s_dist[s_top] > 0.80:
                    highlights.append({
                        "sentence": s,
                        "sentiment": s_top,
                        "score": s_dist[s_top]
                    })
            
            # Sort highlights by score descending and keep top 2
            highlights.sort(key=lambda x: x["score"], reverse=True)
            highlights = highlights[:2]
            
            # 3. Entity Extraction
            entities = self._extract_entities(text)
            
            return {
                "sentiment": top_sentiment,
                "score": top_score,
                "distribution": dist,
                "highlights": highlights,
                "entities": entities
            }
            
        except Exception as e:
            logger.error(f"Error during advanced sentiment analysis: {e}", exc_info=True)
            return {
                "sentiment": "neutral",
                "score": 0.0,
                "distribution": {"positive": 0.0, "negative": 0.0, "neutral": 1.0},
                "highlights": [],
                "entities": {"tickers": [], "currencies": []}
            }

    def _extract_entities(self, text: str) -> Dict[str, List[str]]:
        tickers = set()
        currencies = set()
        
        # Match stock tickers prefixed with '$' (e.g. $AAPL, $TSLA)
        stock_matches = re.findall(r'\$[A-Z]{1,5}\b', text)
        for m in stock_matches:
            tickers.add(m.upper())
            
        # Match forex currency pairs with slashes (e.g. EUR/USD, GBP/JPY)
        forex_slashes = re.findall(r'\b[A-Z]{3}/[A-Z]{3}\b', text)
        for m in forex_slashes:
            currencies.add(m.upper())
            
        # Match currency pair combinations written together (e.g. EURUSD, USDJPY)
        common_currencies = r'\b(EUR|USD|GBP|JPY|AUD|CAD|CHF|NZD)(USD|JPY|GBP|EUR|CAD|CHF|AUD|NZD)\b'
        forex_merges = re.findall(common_currencies, text)
        for m1, m2 in forex_merges:
            if m1 != m2:
                currencies.add(f"{m1.upper()}/{m2.upper()}")
                
        # Match common crypto tickers (e.g. BTC, ETH, SOL) in uppercase
        crypto_keywords = r'\b(BTC|ETH|SOL|ADA|XRP|DOT|DOGE|LTC|LINK|SHIB|AVAX|MATIC)\b'
        crypto_matches = re.findall(crypto_keywords, text)
        for m in crypto_matches:
            tickers.add(m.upper())
            
        return {
            "tickers": list(tickers),
            "currencies": list(currencies)
        }
