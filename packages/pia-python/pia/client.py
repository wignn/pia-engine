"""Official PIA SDK - Main Client Facades (PiaClient & AsyncPiaClient)."""

from __future__ import annotations

import logging
from typing import Any, Dict, Optional

import httpx

from .config import PiaConfig
from .logger import setup_logger
from .realtime import AsyncRealtimeClient, RealtimeClient
from .resources.market import AsyncMarketResource, MarketResource
from .resources.news import AsyncNewsResource, NewsResource
from .resources.social import AsyncSocialResource, SocialResource
from .resources.ws import AsyncWsResource, WsResource
from .transport import AsyncTransport, SyncTransport
from .types import RateLimitInfo


class PiaClient:
    """Official synchronous client for the PIA Financial & Market Intelligence Platform.

    Example:
    ```python
    from pia import PiaClient

    client = PiaClient(api_key="wi_live_...")
    prices = client.market.get_prices()
    for item in prices.items:
        print(item.symbol, item.price)
    ```
    """

    def __init__(
        self,
        api_key: Optional[str] = None,
        *,
        base_url: Optional[str] = None,
        ws_url: Optional[str] = None,
        timeout: Optional[float] = None,
        max_retries: Optional[int] = None,
        retry_delay: Optional[float] = None,
        headers: Optional[Dict[str, str]] = None,
        debug: bool = False,
        http_client: Optional[httpx.Client] = None,
        logger: Optional[logging.Logger] = None,
    ) -> None:
        self.config = PiaConfig.resolve(
            api_key=api_key,
            base_url=base_url,
            ws_url=ws_url,
            timeout=timeout,
            max_retries=max_retries,
            retry_delay=retry_delay,
            headers=headers,
            debug=debug,
        )
        self.logger = logger or setup_logger(self.config.debug)
        self._transport = SyncTransport(self.config, client=http_client, logger=self.logger)

        self.market = MarketResource(self._transport)
        self.social = SocialResource(self._transport)
        self.news = NewsResource(self._transport)
        self.ws = WsResource(self._transport)
        self.realtime = RealtimeClient(self.config, logger=self.logger)

    def get_rate_limit_info(self) -> RateLimitInfo:
        """Returns the most recent rate limit and daily quota telemetry."""
        return self._transport.get_rate_limit_info()

    def close(self) -> None:
        self._transport.close()
        self.realtime.stop()

    def __enter__(self) -> PiaClient:
        return self

    def __exit__(self, exc_type: Any, exc_val: Any, exc_tb: Any) -> None:
        self.close()


class AsyncPiaClient:
    """Official asynchronous client for the PIA Financial & Market Intelligence Platform.

    Example:
    ```python
    import asyncio
    from pia import AsyncPiaClient

    async def main():
        async with AsyncPiaClient(api_key="wi_live_...") as client:
            prices = await client.market.get_prices()
            print(f"Tracked assets: {prices.total}")

    asyncio.run(main())
    ```
    """

    def __init__(
        self,
        api_key: Optional[str] = None,
        *,
        base_url: Optional[str] = None,
        ws_url: Optional[str] = None,
        timeout: Optional[float] = None,
        max_retries: Optional[int] = None,
        retry_delay: Optional[float] = None,
        headers: Optional[Dict[str, str]] = None,
        debug: bool = False,
        http_client: Optional[httpx.AsyncClient] = None,
        logger: Optional[logging.Logger] = None,
    ) -> None:
        self.config = PiaConfig.resolve(
            api_key=api_key,
            base_url=base_url,
            ws_url=ws_url,
            timeout=timeout,
            max_retries=max_retries,
            retry_delay=retry_delay,
            headers=headers,
            debug=debug,
        )
        self.logger = logger or setup_logger(self.config.debug)
        self._transport = AsyncTransport(self.config, client=http_client, logger=self.logger)

        self.market = AsyncMarketResource(self._transport)
        self.social = AsyncSocialResource(self._transport)
        self.news = AsyncNewsResource(self._transport)
        self.ws = AsyncWsResource(self._transport)
        self.realtime = AsyncRealtimeClient(self.config, logger=self.logger)

    def get_rate_limit_info(self) -> RateLimitInfo:
        """Returns the most recent rate limit and daily quota telemetry."""
        return self._transport.get_rate_limit_info()

    async def aclose(self) -> None:
        await self._transport.aclose()
        await self.realtime.close()

    async def __aenter__(self) -> AsyncPiaClient:
        return self

    async def __aexit__(self, exc_type: Any, exc_val: Any, exc_tb: Any) -> None:
        await self.aclose()
