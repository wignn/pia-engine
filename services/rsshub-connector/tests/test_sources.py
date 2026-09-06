import asyncio
import unittest

from app.config import SourceAccount, parse_accounts
from app.sources.truth import TruthSocialSource


class SourceTests(unittest.TestCase):
    def test_parse_rsshub_accounts(self):
        accounts = parse_accounts("rsshub:Alice, rsshub:@coinbureau, rsshub:alice")
        self.assertEqual(accounts, (SourceAccount("rsshub", "alice"), SourceAccount("rsshub", "coinbureau")))

    def test_reject_unknown_platform(self):
        with self.assertRaises(ValueError):
            parse_accounts("instagram:someone")

    def test_truth_adapter_is_explicitly_unconfigured(self):
        async def run():
            with self.assertRaises(RuntimeError):
                await TruthSocialSource().fetch_latest(SourceAccount("truth", "realdonaldtrump"), 10)
        asyncio.run(run())


if __name__ == "__main__":
    unittest.main()
