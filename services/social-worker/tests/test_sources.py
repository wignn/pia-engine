import asyncio
import unittest
from unittest.mock import patch

from app.config import SourceAccount, parse_accounts
from app.sources.truth import TruthSocialSource
from app.sources.twitter import TwitterSource


class FakePool:
    def __init__(self):
        self.calls = []

    async def add_account_cookies(self, username, cookies):
        self.calls.append((username, cookies))


class FakeAPI:
    def __init__(self):
        self.pool = FakePool()

    async def user_by_login(self, username):
        return type("User", (), {"id": 1})()

    async def user_tweets(self, user_id, limit):
        if False:
            yield None


class SourceTests(unittest.TestCase):
    def test_parse_multi_platform_accounts(self):
        accounts = parse_accounts("twitter:Alice, truth:@realDonaldTrump, twitter:alice")
        self.assertEqual(accounts, (SourceAccount("twitter", "alice"), SourceAccount("truth", "realdonaldtrump")))

    def test_reject_unknown_platform(self):
        with self.assertRaises(ValueError):
            parse_accounts("instagram:someone")

    def test_truth_adapter_is_explicitly_unconfigured(self):
        async def run():
            with self.assertRaises(RuntimeError):
                await TruthSocialSource().fetch_latest(SourceAccount("truth", "realdonaldtrump"), 10)
        asyncio.run(run())

    def test_twitter_bootstraps_cookies_once(self):
        async def run():
            source = TwitterSource("unused.db", "auth-token", "csrf-token", "x-worker")
            fake_api = FakeAPI()
            with patch("app.sources.twitter.API", return_value=fake_api):
                source = TwitterSource("unused.db", "auth-token", "csrf-token", "x-worker")
                await source._ensure_cookies()
                await source._ensure_cookies()
            self.assertEqual(
                fake_api.pool.calls,
                [("x-worker", "auth_token=auth-token; ct0=csrf-token")],
            )
        asyncio.run(run())

    def test_twitter_without_cookies_keeps_existing_session(self):
        async def run():
            source = TwitterSource("unused.db")
            fake_api = FakeAPI()
            with patch("app.sources.twitter.API", return_value=fake_api):
                source = TwitterSource("unused.db")
                await source._ensure_cookies()
            self.assertEqual(fake_api.pool.calls, [])
        asyncio.run(run())


if __name__ == "__main__":
    unittest.main()
