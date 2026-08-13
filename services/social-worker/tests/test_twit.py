import asyncio
import unittest
from datetime import datetime, timezone
from types import SimpleNamespace

from app.config import Config, SourceAccount
from app.models import TweetRecord
from app.normalizer import normalize_twitter_tweet
from app.poller import PollingWorker


def fake_tweet(tweet_id="10"):
    return SimpleNamespace(
        id=int(tweet_id),
        user=SimpleNamespace(username="alice", displayname="Alice"),
        rawContent="hello",
        url=f"https://x.com/alice/status/{tweet_id}",
        date=datetime(2026, 1, 1, tzinfo=timezone.utc),
        replyCount=1,
        retweetCount=2,
        likeCount=3,
        quoteCount=4,
        lang="en",
        media=SimpleNamespace(photos=[], videos=[], animated=[]),
    )


class WorkerTests(unittest.TestCase):
    def test_normalize_tweet(self):
        record = normalize_twitter_tweet(fake_tweet(), "alice")
        self.assertEqual(record.post_id, "10")
        self.assertEqual(record.platform, "twitter")
        self.assertEqual(record.created_at, "2026-01-01T00:00:00Z")

    def test_worker_accepts_non_numeric_post_ids(self):
        async def fetcher(account, limit):
            return [TweetRecord(
                "truth-post", account.platform, account.username, "alice", "Alice",
                "hello", "", "2026-01-01T00:00:00Z", "", 0, 0, 0, 0,
            )]

        async def run():
            account = SourceAccount("truth", "alice")
            worker = PollingWorker(Config((account,), 0.01, 2), fetcher, sources={})
            stream = worker.subscribe()
            await worker.poll_account(account)
            self.assertEqual((await stream.__anext__()).post_id, "truth-post")
            await stream.aclose()

        asyncio.run(run())

    def test_worker_deduplicates_per_platform_account(self):
        async def fetcher(account, limit):
            return [TweetRecord(
                "10", account.platform, account.username, "alice", "Alice",
                "hello", "", "", "", 0, 0, 0, 0,
            )]

        async def run():
            accounts = (SourceAccount("twitter", "alice"), SourceAccount("truth", "alice"))
            worker = PollingWorker(Config(accounts, 0.01, 2), fetcher, sources={})
            stream = worker.subscribe()
            await worker.poll_account(accounts[0])
            await worker.poll_account(accounts[1])
            self.assertEqual((await stream.__anext__()).event_id, "twitter:10")
            self.assertEqual((await stream.__anext__()).event_id, "truth:10")
            await stream.aclose()

        asyncio.run(run())


if __name__ == "__main__":
    unittest.main()
