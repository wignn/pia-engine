from typing import Any, Dict, List, Optional
from pydantic import BaseModel, Field


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


class WhyMoveMove(BaseModel):
    latest_price: Optional[float] = None
    baseline_price: Optional[float] = None
    move_pct: Optional[float] = None
    direction: str = "none"
    severity: Optional[str] = None
    threshold_pct: Optional[float] = None
    tick_count: int = 0
    latest_at: Optional[str] = None
    is_active_spike: bool = False


class WhyMoveNewsItem(BaseModel):
    kind: str = "news"
    title: str = ""
    summary: Optional[str] = None
    source_name: Optional[str] = None
    url: Optional[str] = None
    published_at: Optional[str] = None
    processed_at: Optional[str] = None
    sentiment: Optional[str] = None
    impact_level: Optional[str] = None
    matched_terms: List[str] = Field(default_factory=list)
    score: float = 0.0
    reason: Optional[str] = None


class WhyMoveCrossAsset(BaseModel):
    symbol: str
    asset_type: Optional[str] = None
    move_pct: float = 0.0
    direction: str = "none"
    latest_price: Optional[float] = None
    tick_count: int = 0
    latest_at: Optional[str] = None
    relationship: Optional[str] = None


class WhyMoveRequest(BaseModel):
    symbol: str
    window: str = "5m"
    lookback_minutes: int = 180
    move: Optional[WhyMoveMove] = None
    summary: Optional[str] = None
    confidence: Optional[str] = None
    matched_terms: List[str] = Field(default_factory=list)
    drivers: List[str] = Field(default_factory=list)
    causes: Dict[str, List[WhyMoveNewsItem]] = Field(default_factory=dict)
    cross_assets: List[WhyMoveCrossAsset] = Field(default_factory=list)
    calendar: List[Dict[str, Any]] = Field(default_factory=list)


class WhyMoveDriver(BaseModel):
    name: str
    score: float
    evidence: List[str] = Field(default_factory=list)


class WhyMoveCluster(BaseModel):
    theme: str
    score: float
    sentiment: str = "neutral"
    headlines: List[str] = Field(default_factory=list)


class WhyMoveConfidence(BaseModel):
    label: str
    score: float
    breakdown: Dict[str, float]


class WhyMoveNarrative(BaseModel):
    headline: str
    explanation: str
    drivers: List[str] = Field(default_factory=list)
    confidence: str = "low"
    caveats: List[str] = Field(default_factory=list)


class WhyMoveEngineStatus(BaseModel):
    provider: str = "deterministic"
    model: Optional[str] = None
    status: str = "disabled"
    narrative: Optional[WhyMoveNarrative] = None


class WhyMoveResponse(BaseModel):
    symbol: str
    window: str
    move: Optional[WhyMoveMove] = None
    headline: str
    explanation: str
    drivers: List[WhyMoveDriver]
    news_clusters: List[WhyMoveCluster]
    confidence: WhyMoveConfidence
    caveats: List[str] = Field(default_factory=list)
    causes: Dict[str, List[WhyMoveNewsItem]] = Field(default_factory=dict)
    cross_assets: List[WhyMoveCrossAsset] = Field(default_factory=list)
    llm: WhyMoveEngineStatus


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
