"""Unit tests for PiaConfig resolution and validation."""

import os
import unittest

from pia import ConfigurationError, PiaConfig


class TestPiaConfig(unittest.TestCase):
    def test_default_resolution_with_explicit_key(self):
        config = PiaConfig.resolve(api_key="wi_live_explicit_key")
        self.assertEqual(config.api_key, "wi_live_explicit_key")
        self.assertEqual(config.base_url, "https://api-engine.wign.dev")
        self.assertEqual(config.ws_url, "wss://api-engine.wign.dev/api/v1/ws")
        self.assertEqual(config.timeout, 15.0)
        self.assertEqual(config.max_retries, 3)

    def test_missing_api_key_raises_configuration_error(self):
        old_env = os.environ.get("PIA_API_KEY")
        if "PIA_API_KEY" in os.environ:
            del os.environ["PIA_API_KEY"]
        if "ATSLD_API_KEY" in os.environ:
            del os.environ["ATSLD_API_KEY"]
        if "CORE_API_KEY" in os.environ:
            del os.environ["CORE_API_KEY"]

        try:
            with self.assertRaises(ConfigurationError):
                PiaConfig.resolve()
        finally:
            if old_env:
                os.environ["PIA_API_KEY"] = old_env

    def test_environment_variable_fallback(self):
        os.environ["PIA_API_KEY"] = "wi_live_from_env_var"
        try:
            config = PiaConfig.resolve()
            self.assertEqual(config.api_key, "wi_live_from_env_var")
        finally:
            del os.environ["PIA_API_KEY"]

    def test_strips_trailing_slashes(self):
        config = PiaConfig.resolve(
            api_key="test",
            base_url="https://custom.wign.dev///",
            ws_url="wss://custom.wign.dev/ws///",
        )
        self.assertEqual(config.base_url, "https://custom.wign.dev")
        self.assertEqual(config.ws_url, "wss://custom.wign.dev/ws")

    def test_invalid_urls_raise_error(self):
        with self.assertRaises(ConfigurationError):
            PiaConfig.resolve(api_key="test", base_url="ftp://invalid")

        with self.assertRaises(ConfigurationError):
            PiaConfig.resolve(api_key="test", ws_url="http://invalid-ws-proto")

    def test_negative_retries_or_invalid_timeout(self):
        with self.assertRaises(ConfigurationError):
            PiaConfig.resolve(api_key="test", max_retries=-1)

        with self.assertRaises(ConfigurationError):
            PiaConfig.resolve(api_key="test", timeout=0.01)


if __name__ == "__main__":
    unittest.main()
