"""Unit tests for Social, News, and Ws resources."""

import unittest
import httpx

from pia import PiaClient


class TestOtherResources(unittest.TestCase):
    def test_social_posts_resource(self):
        def mock_handler(request: httpx.Request) -> httpx.Response:
            self.assertIn("/api/v1/social/posts", str(request.url))
            self.assertIn("symbol=BTC", str(request.url))
            return httpx.Response(
                200,
                json={
                    "total": 1,
                    "posts": [
                        {
                            "id": "post-1",
                            "author": "Trader",
                            "author_handle": "trader1",
                            "content": "Bitcoin looking bullish",
                            "posted_at": "2026-09-10T10:00:00Z",
                            "symbols": ["BTC"],
                        }
                    ],
                },
            )

        client = PiaClient(
            api_key="test",
            http_client=httpx.Client(transport=httpx.MockTransport(mock_handler)),
        )
        res = client.social.get_posts(symbol="btc")
        self.assertEqual(res.total, 1)
        self.assertEqual(res.posts[0].author_handle, "trader1")

    def test_news_resource(self):
        def mock_handler(request: httpx.Request) -> httpx.Response:
            self.assertIn("/api/v1/news", str(request.url))
            self.assertIn("symbols=XAUUSD%2CBTC", str(request.url))
            return httpx.Response(
                200,
                json={
                    "total": 1,
                    "items": [
                        {
                            "id": "news-1",
                            "title": "Gold reaches new highs",
                            "summary": "Market analysis on gold...",
                            "url": "https://news.wign.dev/1",
                            "source": "Bloomberg",
                            "published_at": "2026-09-10T10:00:00Z",
                        }
                    ],
                },
            )

        client = PiaClient(
            api_key="test",
            http_client=httpx.Client(transport=httpx.MockTransport(mock_handler)),
        )
        res = client.news.get_news(symbols=["xauusd", "btc"])
        self.assertEqual(res.total, 1)
        self.assertEqual(res.items[0].source, "Bloomberg")

    def test_ws_ticket_resource(self):
        def mock_handler(request: httpx.Request) -> httpx.Response:
            self.assertIn("/api/v1/ws/ticket", str(request.url))
            self.assertEqual(request.method, "POST")
            return httpx.Response(
                200,
                json={"ticket": "wst_abcdef12345", "expires_in": 60},
            )

        client = PiaClient(
            api_key="test",
            http_client=httpx.Client(transport=httpx.MockTransport(mock_handler)),
        )
        res = client.ws.create_ticket()
        self.assertEqual(res.ticket, "wst_abcdef12345")
        self.assertEqual(res.expires_in, 60)


if __name__ == "__main__":
    unittest.main()
