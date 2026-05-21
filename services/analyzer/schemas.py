from typing import Any, Dict, List, Optional
from pydantic import BaseModel


class AnalysisRequest(BaseModel):
    text: Optional[str] = None
    url: Optional[str] = None


class EntityResponse(BaseModel):
    tickers: List[str]
    currencies: List[str]


class HighlightItem(BaseModel):
    sentence: str
    sentiment: str
    score: float


class EventResponse(BaseModel):
    type: Optional[str] = None
    country: Optional[str] = None
    actual: Optional[float] = None
    forecast: Optional[float] = None
    previous: Optional[float] = None
    unit: Optional[str] = None
    raw: Optional[Dict[str, Any]] = None


class AnalysisResponse(BaseModel):
    # Existing response fields. Keep these so old clients do not break.
    sentiment: str
    score: float
    distribution: Dict[str, float]
    highlights: List[HighlightItem]
    entities: EntityResponse
    title: Optional[str] = None
    content: Optional[str] = None

    # New optional fields for trading/news intelligence.
    language: Optional[str] = None
    model_used: Optional[str] = None
    event: Optional[EventResponse] = None
    market_impact: Optional[Dict[str, str]] = None
    final_signal: Optional[str] = None
    reason: Optional[str] = None
    confidence: Optional[float] = None

    # Translation layer fields. Old clients can ignore these.
    translated: Optional[bool] = None
    translated_text: Optional[str] = None
    translation_error: Optional[str] = None
    analysis_language: Optional[str] = None
