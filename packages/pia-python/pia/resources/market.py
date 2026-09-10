"""Official PIA SDK - Market Intelligence Resource."""

from __future__ import annotations

from typing import Any, Dict, Optional, Union
from urllib.parse import quote

from ..errors import ValidationError
from ..transport import AsyncTransport, SyncTransport
from ..types import CandleResponse, MarketPricesResponse, OrderBook


def _validate_symbol(symbol: str) -> str:
    if not isinstance(symbol, str) or not symbol.strip():
        raise ValidationError("Symbol must be a non-empty string.", param_name="symbol")
    return symbol.strip().upper()


class MarketResource:
    """Synchronous Market Data API resource."""

    def __init__(self, transport: SyncTransport) -> None:
        self._transport = transport

    def get_prices(self) -> MarketPricesResponse:
        """Retrieves a live price snapshot for all tracked global financial instruments."""
        data = self._transport.request("/api/v1/market/prices", method="GET")
        return MarketPricesResponse.from_dict(data)

    def get_candles(
        self,
        symbol: str,
        *,
        timeframe: str = "1h",
        limit: Optional[int] = None,
        since: Optional[int] = None,
        until: Optional[int] = None,
    ) -> CandleResponse:
        """Fetches historical candlestick bars for a specific instrument and timeframe."""
        clean_symbol = _validate_symbol(symbol)
        params: Dict[str, Any] = {"resolution": timeframe}
        if limit is not None:
            params["limit"] = limit
        if since is not None:
            params["since"] = since
        if until is not None:
            params["before"] = until

        endpoint = f"/api/v1/market/history/{quote(clean_symbol)}"
        data = self._transport.request(endpoint, method="GET", params=params)
        return CandleResponse.from_dict(data, symbol=clean_symbol, timeframe=timeframe)

    def get_order_book(self, symbol: str) -> OrderBook:
        """Retrieves Level 2 Depth of Market (DOM) order book for an instrument."""
        clean_symbol = _validate_symbol(symbol)
        endpoint = f"/api/v1/market/orderbook/{quote(clean_symbol)}"
        data = self._transport.request(endpoint, method="GET")
        return OrderBook.from_dict(data)


class AsyncMarketResource:
    """Asynchronous Market Data API resource."""

    def __init__(self, transport: AsyncTransport) -> None:
        self._transport = transport

    async def get_prices(self) -> MarketPricesResponse:
        """Retrieves a live price snapshot for all tracked global financial instruments."""
        data = await self._transport.request("/api/v1/market/prices", method="GET")
        return MarketPricesResponse.from_dict(data)

    async def get_candles(
        self,
        symbol: str,
        *,
        timeframe: str = "1h",
        limit: Optional[int] = None,
        since: Optional[int] = None,
        until: Optional[int] = None,
    ) -> CandleResponse:
        """Fetches historical candlestick bars for a specific instrument and timeframe."""
        clean_symbol = _validate_symbol(symbol)
        params: Dict[str, Any] = {"resolution": timeframe}
        if limit is not None:
            params["limit"] = limit
        if since is not None:
            params["since"] = since
        if until is not None:
            params["before"] = until

        endpoint = f"/api/v1/market/history/{quote(clean_symbol)}"
        data = await self._transport.request(endpoint, method="GET", params=params)
        return CandleResponse.from_dict(data, symbol=clean_symbol, timeframe=timeframe)

    async def get_order_book(self, symbol: str) -> OrderBook:
        """Retrieves Level 2 Depth of Market (DOM) order book for an instrument."""
        clean_symbol = _validate_symbol(symbol)
        endpoint = f"/api/v1/market/orderbook/{quote(clean_symbol)}"
        data = await self._transport.request(endpoint, method="GET")
        return OrderBook.from_dict(data)
