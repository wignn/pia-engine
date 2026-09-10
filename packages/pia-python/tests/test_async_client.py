"""Unit tests for AsyncPiaClient."""

import unittest
import httpx

from pia import AsyncPiaClient


class TestAsyncPiaClient(unittest.IsolatedAsyncioTestCase):
    async def test_async_get_prices_and_telemetry(self):
        def mock_handler(request: httpx.Request) -> httpx.Response:
            return httpx.Response(
                200,
                json={
                    "total": 1,
                    "timestamp": 1726000000,
                    "items": [{"symbol": "XAUUSD", "price": 2510.0, "bid": 2509.9, "ask": 2510.1}],
                },
                headers={
                    "x-ratelimit-remaining": "55",
                    "x-dailyquota-remaining": "990",
                },
            )

        async with AsyncPiaClient(
            api_key="test",
            http_client=httpx.AsyncClient(transport=httpx.MockTransport(mock_handler)),
        ) as client:
            res = await client.market.get_prices()
            self.assertEqual(res.total, 1)
            self.assertEqual(res.items[0].symbol, "XAUUSD")

            telemetry = client.get_rate_limit_info()
            self.assertEqual(telemetry.remaining, 55)
            self.assertEqual(telemetry.daily_remaining, 990)


if __name__ == "__main__":
    unittest.main()
