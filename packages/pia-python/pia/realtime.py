"""Official PIA SDK - Resilient Realtime WebSocket Client with In-Band Auth."""

from __future__ import annotations

import asyncio
import json
import logging
import random
import threading
from typing import Any, AsyncIterator, Callable, Dict, List, Optional, Set, Union

from .config import PiaConfig
from .errors import AuthenticationError, NetworkError
from .logger import setup_logger
from .types import MarketPrice

try:
    import websockets
    import websockets.client
    HAS_WEBSOCKETS = True
except ImportError:
    HAS_WEBSOCKETS = False


class AsyncRealtimeClient:
    """Asynchronous WebSocket client using asyncio and websockets."""

    def __init__(self, config: PiaConfig, logger: Optional[logging.Logger] = None) -> None:
        if not HAS_WEBSOCKETS:
            raise ImportError(
                "The 'websockets' library is required for realtime streaming. "
                "Install it with: pip install 'pia-sdk[realtime]' or pip install websockets"
            )
        self.config = config
        self.logger = logger or setup_logger(config.debug)
        self.subscribed_symbols: Set[str] = set()
        self._ws: Any = None
        self._is_authenticated = False
        self._is_closed = False
        self._handlers: Dict[str, List[Callable[..., Any]]] = {
            "connect": [],
            "authenticated": [],
            "tick": [],
            "snapshot": [],
            "disconnect": [],
            "error": [],
        }

    def on(self, event: str, handler: Callable[..., Any]) -> AsyncRealtimeClient:
        """Registers an event callback (e.g. 'tick', 'authenticated', 'error')."""
        if event in self._handlers:
            self._handlers[event].append(handler)
        return self

    def _emit(self, event: str, *args: Any) -> None:
        for handler in self._handlers.get(event, []):
            try:
                res = handler(*args)
                if asyncio.iscoroutine(res):
                    asyncio.create_task(res)
            except Exception as err:
                self.logger.error("Error in realtime event handler for '%s': %s", event, err)

    def subscribe(self, symbols: Union[str, List[str]]) -> AsyncRealtimeClient:
        """Adds symbols to the active subscription set."""
        sym_list = [symbols] if isinstance(symbols, str) else symbols
        for s in sym_list:
            clean = s.strip().upper()
            if clean:
                self.subscribed_symbols.add(clean)

        if self._is_authenticated and self._ws:
            asyncio.create_task(self._send_subscribe(sym_list))
        return self

    async def _send_subscribe(self, symbols: List[str]) -> None:
        if self._ws:
            payload = {"action": "subscribe", "symbols": [s.strip().upper() for s in symbols if s.strip()]}
            await self._ws.send(json.dumps(payload))

    async def stream(self, symbols: Optional[List[str]] = None) -> AsyncIterator[MarketPrice]:
        """Asynchronous generator that yields real-time market ticks."""
        if symbols:
            self.subscribe(symbols)

        queue: asyncio.Queue[MarketPrice] = asyncio.Queue()

        def _on_tick(tick: MarketPrice) -> None:
            queue.put_nowait(tick)

        self.on("tick", _on_tick)
        task = asyncio.create_task(self.listen())

        try:
            while not self._is_closed:
                tick = await queue.get()
                yield tick
        finally:
            task.cancel()

    async def listen(self) -> None:
        """Connects and maintains a persistent realtime stream with auto-reconnect."""
        self._is_closed = False
        reconnect_attempt = 0

        while not self._is_closed:
            try:
                self.logger.info("Connecting to WebSocket: %s", self.config.ws_url)
                async with websockets.connect(
                    self.config.ws_url,
                    ping_interval=30,
                    ping_timeout=10,
                ) as ws:
                    self._ws = ws
                    self._is_authenticated = False
                    reconnect_attempt = 0
                    self._emit("connect")

                    # 1. In-Band Message Authentication
                    auth_frame = {"action": "auth", "api_key": self.config.api_key}
                    await ws.send(json.dumps(auth_frame))

                    async for message in ws:
                        if self._is_closed:
                            break
                        await self._handle_raw_message(message)

            except asyncio.CancelledError:
                break
            except Exception as err:
                self.logger.warning("WebSocket dropped: %s", err)
                self._emit("disconnect", str(err))

                if self._is_closed:
                    break

                reconnect_attempt += 1
                delay = min(1.0 * (1.5 ** (reconnect_attempt - 1)), 15.0) * (0.8 + random.random() * 0.4)
                self.logger.info("Reconnecting in %.2fs (Attempt %d)...", delay, reconnect_attempt)
                await asyncio.sleep(delay)

    async def _handle_raw_message(self, raw_message: Union[str, bytes]) -> None:
        try:
            payload = json.loads(raw_message)
        except Exception:
            return

        event = payload.get("event")

        # 1. Auth confirmation
        if event == "authenticated":
            self._is_authenticated = True
            self.logger.info("Realtime session authenticated successfully.")
            self._emit("authenticated", payload.get("data", {}))

            if self.subscribed_symbols:
                await self._send_subscribe(list(self.subscribed_symbols))
            return

        # 2. Market price tick
        if event in ("market.trade", "market.tick"):
            tick_dict = payload.get("data", {}).get("tick") or payload.get("data")
            if isinstance(tick_dict, dict):
                tick = MarketPrice.from_dict(tick_dict)
                self._emit("tick", tick)
            return

        # 3. Snapshot
        if event == "market.snapshot":
            self._emit("snapshot", payload.get("data", {}))
            return

        # 4. Error payload
        if payload.get("error"):
            err_code = payload.get("error")
            msg = payload.get("message", "Realtime stream error")
            if err_code in ("unauthorized", 401):
                self._emit("error", AuthenticationError(msg))
            else:
                self._emit("error", NetworkError(msg))

    async def close(self) -> None:
        """Closes the realtime stream connection."""
        self._is_closed = True
        if self._ws:
            await self._ws.close()


class RealtimeClient:
    """Threaded/Callback Realtime client for synchronous applications."""

    def __init__(self, config: PiaConfig, logger: Optional[logging.Logger] = None) -> None:
        self._config = config
        self._logger = logger or setup_logger(config.debug)
        self._async_client = AsyncRealtimeClient(config, self._logger)
        self._thread: Optional[threading.Thread] = None
        self._loop: Optional[asyncio.AbstractEventLoop] = None

    def on(self, event: str, handler: Callable[..., Any]) -> RealtimeClient:
        self._async_client.on(event, handler)
        return self

    def subscribe(self, symbols: Union[str, List[str]]) -> RealtimeClient:
        self._async_client.subscribe(symbols)
        return self

    def start(self) -> RealtimeClient:
        """Starts the WebSocket listener in a background daemon thread."""
        if self._thread and self._thread.is_alive():
            return self

        def _runner() -> None:
            self._loop = asyncio.new_event_loop()
            asyncio.set_event_loop(self._loop)
            self._loop.run_until_complete(self._async_client.listen())

        self._thread = threading.Thread(target=_runner, daemon=True, name="pia-realtime")
        self._thread.start()
        return self

    def stop(self) -> None:
        """Stops the background WebSocket listener."""
        if self._loop and self._loop.is_running():
            asyncio.run_coroutine_threadsafe(self._async_client.close(), self._loop)
        if self._thread:
            self._thread.join(timeout=2.0)
            self._thread = None

    def run_forever(self) -> None:
        """Blocks the current thread and runs the WebSocket listener."""
        asyncio.run(self._async_client.listen())
