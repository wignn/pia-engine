"""Unit tests for Realtime streaming In-Band Auth and event dispatch."""

import asyncio
import json
import unittest

from pia import MarketPrice, PiaConfig
from pia.realtime import AsyncRealtimeClient


class TestRealtimeClient(unittest.IsolatedAsyncioTestCase):
    async def test_in_band_auth_and_tick_processing(self):
        config = PiaConfig.resolve(api_key="wi_live_testkey123")
        client = AsyncRealtimeClient(config)

        auth_received = False
        captured_tick: MarketPrice | None = None

        def on_auth(info):
            nonlocal auth_received
            auth_received = True

        def on_tick(tick: MarketPrice):
            nonlocal captured_tick
            captured_tick = tick

        client.on("authenticated", on_auth)
        client.on("tick", on_tick)

        # Simulate receiving authenticated event
        auth_msg = json.dumps({"event": "authenticated", "data": {"plan": "starter"}})
        await client._handle_raw_message(auth_msg)
        self.assertTrue(auth_received)

        # Simulate receiving market tick event
        tick_msg = json.dumps({
            "event": "market.trade",
            "data": {
                "tick": {
                    "symbol": "XAUUSD",
                    "price": 2515.50,
                    "bid": 2515.40,
                    "ask": 2515.60,
                }
            }
        })
        await client._handle_raw_message(tick_msg)

        self.assertIsNotNone(captured_tick)
        self.assertEqual(captured_tick.symbol, "XAUUSD")
        self.assertEqual(captured_tick.price, 2515.50)

    async def test_symbol_subscription_queue(self):
        config = PiaConfig.resolve(api_key="key")
        client = AsyncRealtimeClient(config)

        client.subscribe(["XAUUSD", "btcusdt", "   "])
        self.assertIn("XAUUSD", client.subscribed_symbols)
        self.assertIn("BTCUSDT", client.subscribed_symbols)
        self.assertEqual(len(client.subscribed_symbols), 2)


if __name__ == "__main__":
    unittest.main()
