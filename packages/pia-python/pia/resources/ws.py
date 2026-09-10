"""Official PIA SDK - WebSocket Ephemeral Ticket Resource."""

from __future__ import annotations

from ..transport import AsyncTransport, SyncTransport
from ..types import WsTicketResponse


class WsResource:
    """Synchronous WebSocket Handshake Ticket resource."""

    def __init__(self, transport: SyncTransport) -> None:
        self._transport = transport

    def create_ticket(self) -> WsTicketResponse:
        """Issues a single-use 60-second WebSocket connection ticket."""
        data = self._transport.request("/api/v1/ws/ticket", method="POST", json_data={})
        return WsTicketResponse.from_dict(data)


class AsyncWsResource:
    """Asynchronous WebSocket Handshake Ticket resource."""

    def __init__(self, transport: AsyncTransport) -> None:
        self._transport = transport

    async def create_ticket(self) -> WsTicketResponse:
        """Issues a single-use 60-second WebSocket connection ticket."""
        data = await self._transport.request("/api/v1/ws/ticket", method="POST", json_data={})
        return WsTicketResponse.from_dict(data)
