"""Unit tests for PiaError hierarchy and sensitive data redaction."""

import unittest

from pia import (
    ApiError,
    AuthenticationError,
    ConfigurationError,
    NetworkError,
    ParseError,
    PermissionError,
    PiaError,
    RateLimitError,
    TimeoutError,
    ValidationError,
    redact_sensitive,
)


class TestPiaErrors(unittest.TestCase):
    def test_error_hierarchy_inheritance(self):
        errors = [
            ConfigurationError("bad config"),
            ValidationError("bad input", param_name="symbol"),
            AuthenticationError("bad key"),
            PermissionError("forbidden", required_scope="realtime:ws"),
            RateLimitError("quota exhausted"),
            TimeoutError("timed out", timeout_seconds=10.0),
            NetworkError("dns error"),
            ParseError("json decode fail"),
            ApiError("internal error", status_code=500),
        ]

        for err in errors:
            self.assertIsInstance(err, Exception)
            self.assertIsInstance(err, PiaError)

    def test_rate_limit_error_properties(self):
        err = RateLimitError(
            "Limit reached",
            retry_after_seconds=30,
            daily_limit=1000,
            daily_remaining=0,
            minute_limit=60,
            minute_remaining=0,
        )
        self.assertEqual(err.retry_after_seconds, 30)
        self.assertEqual(err.daily_remaining, 0)
        self.assertEqual(err.minute_remaining, 0)

    def test_sensitive_redaction(self):
        raw = "Auth failed with wi_live_abcdef0123456789 and Bearer secret_bearer_token"
        sanitized = redact_sensitive(raw)

        self.assertNotIn("wi_live_abcdef0123456789", sanitized)
        self.assertNotIn("secret_bearer_token", sanitized)
        self.assertIn("wi_live_***6789", sanitized)
        self.assertIn("Bearer [REDACTED]", sanitized)

    def test_error_str_representation_redacts_keys(self):
        err = AuthenticationError("Key wi_live_998877665544332211 is invalid")
        self.assertNotIn("wi_live_998877665544332211", str(err))
        self.assertIn("wi_live_***2211", str(err))


if __name__ == "__main__":
    unittest.main()
