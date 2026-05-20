import os
import logging
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from typing import Dict, Any, List

from analyzer import AdvancedSentimentAnalyzer

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(name)s - %(message)s"
)
logger = logging.getLogger("finbert-service")

app = FastAPI(
    title="Advanced FinBERT Sentiment API",
    description="Dedicated microservice for deep NLP analysis of financial documents/news.",
    version="2.0.0"
)

# Instantiate the analyzer
analyzer = AdvancedSentimentAnalyzer()

class AnalysisRequest(BaseModel):
    text: str

class EntityResponse(BaseModel):
    tickers: List[str]
    currencies: List[str]

class HighlightItem(BaseModel):
    sentence: str
    sentiment: str
    score: float

class AnalysisResponse(BaseModel):
    sentiment: str
    score: float
    distribution: Dict[str, float]
    highlights: List[HighlightItem]
    entities: EntityResponse

@app.on_event("startup")
def startup_event():
    # Warm up / Load the model on startup
    analyzer.initialize()
    logger.info("Service initialized and ready to process requests.")

@app.get("/health")
def health():
    return {"status": "ready" if analyzer.pipeline is not None else "initializing"}

@app.post("/analyze", response_model=AnalysisResponse)
def analyze(request: AnalysisRequest):
    result = analyzer.analyze(request.text)
    return result

if __name__ == "__main__":
    import uvicorn
    port = int(os.getenv("PORT", 5000))
    logger.info(f"Starting server on port {port}...")
    uvicorn.run("main:app", host="0.0.0.0", port=port, reload=False)
