import os
import unittest
from unittest.mock import patch

from app.config import Config


class ConfigTests(unittest.TestCase):
    def test_nats_config_and_accounts(self):
        env = {
            "SOCIAL_ACCOUNTS": "twitter:@Alice,truth:bob,twitter:alice",
            "X_AUTH_TOKEN": "auth-token",
            "X_CT0": "csrf-token",
            "X_ACCOUNT_NAME": "x-worker",
            "NATS_URL": "nats://broker:4222",
            "NATS_SUBJECT": "tweets.new",
        }
        with patch.dict(os.environ, env, clear=True):
            config = Config.from_env()
        self.assertEqual(config.accounts[0].key, "twitter:alice")
        self.assertEqual(config.accounts[1].key, "truth:bob")
        self.assertEqual(config.x_auth_token, "auth-token")
        self.assertEqual(config.x_ct0, "csrf-token")
        self.assertEqual(config.x_account_name, "x-worker")
        self.assertEqual(config.nats_url, "nats://broker:4222")
        self.assertEqual(config.nats_subject, "tweets.new")

    def test_accounts_required(self):
        with patch.dict(os.environ, {"SOCIAL_ACCOUNTS": "", "TWITTER_ACCOUNTS": ""}, clear=True):
            with self.assertRaises(ValueError):
                Config.from_env()

    def test_interval_positive(self):
        with patch.dict(os.environ, {"SOCIAL_ACCOUNTS": "twitter:alice", "TWITTER_POLL_SECONDS": "0"}, clear=True):
            with self.assertRaises(ValueError):
                Config.from_env()

    def test_x_cookies_must_be_a_pair(self):
        with patch.dict(
            os.environ,
            {"SOCIAL_ACCOUNTS": "twitter:alice", "X_AUTH_TOKEN": "auth-token", "X_CT0": ""},
            clear=True,
        ):
            with self.assertRaises(ValueError):
                Config.from_env()

    def test_x_cookies_can_be_absent(self):
        with patch.dict(os.environ, {"SOCIAL_ACCOUNTS": "twitter:alice"}, clear=True):
            config = Config.from_env()
        self.assertEqual(config.x_auth_token, "")
        self.assertEqual(config.x_ct0, "")
        self.assertEqual(config.x_account_name, "worker")


if __name__ == "__main__":
    unittest.main()
