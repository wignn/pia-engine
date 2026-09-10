"""Official PIA SDK - Social Intelligence Resource."""

from __future__ import annotations

from typing import Any, Dict, Optional

from ..transport import AsyncTransport, SyncTransport
from ..types import SocialFeedResponse


class SocialResource:
    """Synchronous Social Sentiment & Discussions resource."""

    def __init__(self, transport: SyncTransport) -> None:
        self._transport = transport

    def get_posts(
        self,
        *,
        symbol: Optional[str] = None,
        limit: Optional[int] = None,
        cursor: Optional[str] = None,
    ) -> SocialFeedResponse:
        """Fetches curated social discussions and market sentiment posts."""
        params: Dict[str, Any] = {}
        if symbol:
            params["symbol"] = symbol.strip().upper()
        if limit is not None:
            params["limit"] = limit
        if cursor:
            params["cursor"] = cursor

        data = self._transport.request("/api/v1/social/posts", method="GET", params=params)
        return SocialFeedResponse.from_dict(data)


class AsyncSocialResource:
    """Asynchronous Social Sentiment & Discussions resource."""

    def __init__(self, transport: AsyncTransport) -> None:
        self._transport = transport

    async def get_posts(
        self,
        *,
        symbol: Optional[str] = None,
        limit: Optional[int] = None,
        cursor: Optional[str] = None,
    ) -> SocialFeedResponse:
        """Fetches curated social discussions and market sentiment posts."""
        params: Dict[str, Any] = {}
        if symbol:
            params["symbol"] = symbol.strip().upper()
        if limit is not None:
            params["limit"] = limit
        if cursor:
            params["cursor"] = cursor

        data = await self._transport.request("/api/v1/social/posts", method="GET", params=params)
        return SocialFeedResponse.from_dict(data)
