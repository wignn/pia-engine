"""Unit tests for MarketResource."""

import unittest
import httpx

from pia import PiaClient, ValidationError


class TestMarketResource(unittest.TestCase):
    def test_get_prices_parses_items(self):
        def mock_handler(request: httpx.Request) -> httpx.Response:
            self.assertIn("/api/v1/market/prices", str(request.url))
            return httpx.Response(
                200,
                json={
                    "total": 2,
                    "timestamp": 1726000000,
                    "items": [
                        {"symbol": "XAUUSD", "price": 2505.5, "bid": 2505.4, "ask": 2505.6},
                        {"symbol": "BTCUSDT", "price": 60500.0, "bid": 60499.0, "ask": 60501.0},
                    ],
                },
            )

        client = PiaClient(
            api_key="test",
            http_client=httpx.Client(transport=httpx.MockTransport(mock_handler)),
        )

        res = client.market.get_prices()
        self.assertEqual(res.total, 2)
        self.assertEqual(len(res.items), 2)
        self.assertEqual(res.items[0].symbol, "XAUUSD")
        self.assertEqual(res.items[1].price, 60500.0)

    def test_get_candles_validates_symbol(self):
        client = PiaClient(
            api_key="test",
            http_client=httpx.Client(transport=httpx.MockTransport(lambda _: httpx.Response(200))),
        )

        with self.assertRaises(ValidationError):
            client.market.get_candles("")

        with self.assertRaises(ValidationError):
            client.market.get_candles("   ")

    def test_get_candles_builds_query_parameters(self):
        requested_url = ""

        def mock_handler(request: httpx.Request) -> httpx.Response:
            nonlocal requested_url
            requested_url = str(request.url)
            return httpx.Response(
                200,
                json={
                    "symbol": "BTCUSDT",
                    "resolution": "1h",
                    "items": [
                        {"time": 1000, "open": 60000, "high": 61000, "low": 59500, "close": 60500, "volume": 120}
                    ],
                },
            )

        client = PiaClient(
            api_key="test",
            http_client=httpx.Client(transport=httpx.MockTransport(mock_handler)),
        )

        res = client.market.get_candles("btcusdt", timeframe="1h", limit=50, since=1720000000)
        self.assertIn("/api/v1/market/history/BTCUSDT", requested_url)
        self.assertIn("resolution=1h", requested_url)
        self.assertIn("limit=50", requested_url)
        self.assertIn("since=1720000000", requested_url)
        self.assertEqual(res.symbol, "BTCUSDT")
        self.assertEqual(len(res.candles), 1)


if __name__ == "__main__":
    unittest.main()
