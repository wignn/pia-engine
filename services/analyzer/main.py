import os
import logging
from contextlib import asynccontextmanager
from typing import Dict, Any, List

from fastapi import FastAPI, HTTPException
from pydantic import BaseModel

from analyzer import AdvancedSentimentAnalyzer

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(name)s - %(message)s",
)
logger = logging.getLogger("finbert-service")

analyzer = AdvancedSentimentAnalyzer()

@asynccontextmanager
async def lifespan(app: FastAPI):
    logger.info("Warming up FinBERT model...")
    analyzer.initialize()
    logger.info("Service initialized and ready to process requests.")
    yield
    logger.info("Service shutting down.")

app = FastAPI(
    title="Advanced FinBERT Sentiment API",
    description="Dedicated microservice for deep NLP analysis of financial documents/news.",
    version="2.1.0",
    lifespan=lifespan,
)
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


@app.get("/health")
def health():
    return {"status": "ready" if analyzer.pipeline is not None else "initializing"}


@app.post("/analyze", response_model=AnalysisResponse)
def analyze(request: AnalysisRequest):
    if not request.text or not request.text.strip():
        raise HTTPException(status_code=422, detail="'text' field must not be empty.")
    result = analyzer.analyze(request.text)
    return result



if __name__ == "__main__":
    import uvicorn
    port = int(os.getenv("PORT", 5000))
    workers = int(os.getenv("WORKERS", 1))   # keep 1 — model is in-process
    logger.info(f"Starting server on port {port} with {workers} worker(s)...")
    uvicorn.run(
        "main:app",
        host="0.0.0.0",
        port=port,
        reload=False,
        workers=workers,
    )