"""Official PIA SDK for Python.

Enterprise market intelligence and realtime financial streaming library.
"""

from .client import AsyncPiaClient, PiaClient
from .config import (
    DEFAULT_BASE_URL,
    DEFAULT_MAX_RETRIES,
    DEFAULT_TIMEOUT_SECONDS,
    DEFAULT_WS_URL,
    PiaConfig,
)
from .errors import (
    ApiError,
    AuthenticationError,
    ConfigurationError,
    NetworkError,
    ParseError,
    PermissionError,
    PiaError,
    RateLimitError,
    TimeoutError,
    ValidationError,
    redact_sensitive,
)
from .realtime import AsyncRealtimeClient, RealtimeClient
from .types import (
    Candle,
    CandleResponse,
    MarketPrice,
    MarketPricesResponse,
    NewsArticle,
    NewsFeedResponse,
    OrderBook,
    OrderBookLevel,
    RateLimitInfo,
    SocialFeedResponse,
    SocialPost,
    WsTicketResponse,
)

__version__ = "1.0.1"

__all__ = [
    "PiaClient",
    "AsyncPiaClient",
    "PiaConfig",
    "PiaError",
    "ConfigurationError",
    "ValidationError",
    "AuthenticationError",
    "PermissionError",
    "RateLimitError",
    "TimeoutError",
    "NetworkError",
    "ParseError",
    "ApiError",
    "redact_sensitive",
    "RealtimeClient",
    "AsyncRealtimeClient",
    "MarketPrice",
    "MarketPricesResponse",
    "Candle",
    "CandleResponse",
    "OrderBook",
    "OrderBookLevel",
    "SocialPost",
    "SocialFeedResponse",
    "NewsArticle",
    "NewsFeedResponse",
    "RateLimitInfo",
    "WsTicketResponse",
    "DEFAULT_BASE_URL",
    "DEFAULT_WS_URL",
    "DEFAULT_TIMEOUT_SECONDS",
    "DEFAULT_MAX_RETRIES",
]
