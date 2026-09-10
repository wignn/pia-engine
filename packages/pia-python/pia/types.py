"""Official PIA SDK - Strongly Typed Data Models."""

from __future__ import annotations

from dataclasses import asdict, dataclass, field
from typing import Any, Dict, List, Optional


@dataclass
class RateLimitInfo:
    """Telemetry information parsed from RFC 6585 and daily quota headers."""

    limit: Optional[int] = None
    remaining: Optional[int] = None
    reset_seconds: Optional[int] = None
    daily_limit: Optional[int] = None
    daily_remaining: Optional[int] = None

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)


@dataclass
class MarketPrice:
    """Snapshot of a financial instrument's current market price and 24h stats."""

    symbol: str
    price: float
    bid: float
    ask: float
    high_24h: Optional[float] = None
    low_24h: Optional[float] = None
    change_24h_pct: Optional[float] = None
    volume_24h: Optional[float] = None
    asset_type: Optional[str] = None
    session_open: Optional[bool] = None
    timestamp: Optional[int] = None

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> MarketPrice:
        p_raw = data.get("price")
        price = float(p_raw) if p_raw is not None else 0.0

        b_raw = data.get("bid")
        bid = float(b_raw) if b_raw is not None else price

        a_raw = data.get("ask")
        ask = float(a_raw) if a_raw is not None else price

        def _float_or_none(val: Any) -> Optional[float]:
            if val is None:
                return None
            try:
                return float(val)
            except (ValueError, TypeError):
                return None

        def _int_or_none(val: Any) -> Optional[int]:
            if val is None:
                return None
            try:
                return int(val)
            except (ValueError, TypeError):
                return None

        return cls(
            symbol=str(data.get("symbol", "")).upper(),
            price=price,
            bid=bid,
            ask=ask,
            high_24h=_float_or_none(data.get("high_24h")),
            low_24h=_float_or_none(data.get("low_24h")),
            change_24h_pct=_float_or_none(data.get("change_24h_pct")),
            volume_24h=_float_or_none(data.get("volume_24h")),
            asset_type=data.get("asset_type"),
            session_open=data.get("session_open"),
            timestamp=_int_or_none(data.get("timestamp")),
        )

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)


@dataclass
class MarketPricesResponse:
    total: int
    timestamp: int
    items: List[MarketPrice] = field(default_factory=list)

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> MarketPricesResponse:
        raw_items = data.get("items") or []
        items = [MarketPrice.from_dict(item) for item in raw_items]
        return cls(
            total=int(data.get("total", len(items))),
            timestamp=int(data.get("timestamp", 0)),
            items=items,
        )

    def to_dict(self) -> Dict[str, Any]:
        return {
            "total": self.total,
            "timestamp": self.timestamp,
            "items": [item.to_dict() for item in self.items],
        }


@dataclass
class Candle:
    """OHLCV historical bar."""

    time: int
    open: float
    high: float
    low: float
    close: float
    volume: float

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> Candle:
        return cls(
            time=int(data.get("time", 0)),
            open=float(data.get("open", 0.0)),
            high=float(data.get("high", 0.0)),
            low=float(data.get("low", 0.0)),
            close=float(data.get("close", 0.0)),
            volume=float(data.get("volume", 0.0)),
        )

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)


@dataclass
class CandleResponse:
    symbol: str
    timeframe: str
    count: int
    candles: List[Candle] = field(default_factory=list)
    has_more: Optional[bool] = None
    next_before: Optional[int] = None

    @classmethod
    def from_dict(cls, data: Dict[str, Any], symbol: str = "", timeframe: str = "") -> CandleResponse:
        raw_candles = data.get("candles") or data.get("items") or []
        candles = [Candle.from_dict(c) for c in raw_candles]
        return cls(
            symbol=str(data.get("symbol") or symbol),
            timeframe=str(data.get("timeframe") or data.get("resolution") or timeframe),
            count=len(candles),
            candles=candles,
            has_more=data.get("has_more"),
            next_before=data.get("next_before"),
        )

    def to_dict(self) -> Dict[str, Any]:
        return {
            "symbol": self.symbol,
            "timeframe": self.timeframe,
            "count": self.count,
            "candles": [c.to_dict() for c in self.candles],
            "has_more": self.has_more,
            "next_before": self.next_before,
        }


@dataclass
class OrderBookLevel:
    price: float
    size: float

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> OrderBookLevel:
        return cls(
            price=float(data.get("price", 0.0)),
            size=float(data.get("size", 0.0)),
        )


@dataclass
class OrderBook:
    symbol: str
    bids: List[OrderBookLevel] = field(default_factory=list)
    asks: List[OrderBookLevel] = field(default_factory=list)
    timestamp: int = 0

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> OrderBook:
        bids = [OrderBookLevel.from_dict(b) for b in (data.get("bids") or [])]
        asks = [OrderBookLevel.from_dict(a) for a in (data.get("asks") or [])]
        return cls(
            symbol=str(data.get("symbol", "")),
            bids=bids,
            asks=asks,
            timestamp=int(data.get("timestamp", 0)),
        )


@dataclass
class SocialPost:
    id: str
    author: str
    author_handle: str
    content: str
    posted_at: str
    media_urls: List[str] = field(default_factory=list)
    sentiment: Optional[str] = None
    symbols: List[str] = field(default_factory=list)

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> SocialPost:
        return cls(
            id=str(data.get("id", "")),
            author=str(data.get("author", "")),
            author_handle=str(data.get("author_handle", "")),
            content=str(data.get("content", "")),
            posted_at=str(data.get("posted_at", "")),
            media_urls=data.get("media_urls") or [],
            sentiment=data.get("sentiment"),
            symbols=data.get("symbols") or [],
        )


@dataclass
class SocialFeedResponse:
    total: int
    posts: List[SocialPost] = field(default_factory=list)
    cursor: Optional[str] = None

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> SocialFeedResponse:
        posts = [SocialPost.from_dict(p) for p in (data.get("posts") or [])]
        return cls(
            total=int(data.get("total", len(posts))),
            posts=posts,
            cursor=data.get("cursor"),
        )


@dataclass
class NewsArticle:
    id: str
    title: str
    summary: str
    url: str
    source: str
    published_at: str
    symbols: List[str] = field(default_factory=list)

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> NewsArticle:
        return cls(
            id=str(data.get("id", "")),
            title=str(data.get("title", "")),
            summary=str(data.get("summary", "")),
            url=str(data.get("url", "")),
            source=str(data.get("source", "")),
            published_at=str(data.get("published_at", "")),
            symbols=data.get("symbols") or [],
        )


@dataclass
class NewsFeedResponse:
    total: int
    items: List[NewsArticle] = field(default_factory=list)

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> NewsFeedResponse:
        items = [NewsArticle.from_dict(i) for i in (data.get("items") or [])]
        return cls(
            total=int(data.get("total", len(items))),
            items=items,
        )


@dataclass
class WsTicketResponse:
    ticket: str
    expires_in: Optional[int] = None
    ws_url: Optional[str] = None

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> WsTicketResponse:
        return cls(
            ticket=str(data.get("ticket", "")),
            expires_in=data.get("expires_in"),
            ws_url=data.get("ws_url"),
        )
