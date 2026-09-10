"""Official PIA SDK - Strongly Typed Exception Hierarchy.

Provides clear, categorized exceptions for validation, authentication,
permissions, rate limits, timeouts, and network failures.
"""

from __future__ import annotations

import re
from typing import Any, Optional


_KEY_PATTERNS = [
    re.compile(r"wi_live_[a-zA-Z0-9_-]{10,}"),
    re.compile(r"Bearer\s+[a-zA-Z0-9._-]+", re.IGNORECASE),
    re.compile(r"api[_-]?key[\"':\s]+[\"']?([a-zA-Z0-9_-]+)[\"']?", re.IGNORECASE),
]


def redact_sensitive(text: str) -> str:
    """Redacts known API keys and tokens from strings to prevent logging leaks."""
    if not isinstance(text, str):
        text = str(text)

    for pattern in _KEY_PATTERNS:
        def _repl(m: re.Match[str]) -> str:
            val = m.group(0)
            if val.startswith("wi_live_"):
                return f"wi_live_***{val[-4:]}"
            if val.lower().startswith("bearer "):
                return "Bearer [REDACTED]"
            return "[REDACTED]"

        text = pattern.sub(_repl, text)
    return text


class PiaError(Exception):
    """Base exception for all errors originating from the PIA SDK."""

    def __init__(
        self,
        message: str,
        *,
        status_code: Optional[int] = None,
        endpoint: Optional[str] = None,
        method: Optional[str] = None,
        request_id: Optional[str] = None,
        raw_response: Any = None,
    ) -> None:
        self.raw_message = message
        self.status_code = status_code
        self.endpoint = endpoint
        self.method = method
        self.request_id = request_id
        self.raw_response = raw_response
        super().__init__(redact_sensitive(message))

    def __str__(self) -> str:
        parts = [redact_sensitive(self.raw_message)]
        if self.status_code:
            parts.append(f"(status: {self.status_code})")
        if self.endpoint:
            parts.append(f"(endpoint: {self.endpoint})")
        if self.request_id:
            parts.append(f"(request_id: {self.request_id})")
        return " ".join(parts)


class ConfigurationError(PiaError):
    """Raised when client options or environment settings are invalid."""


class ValidationError(PiaError):
    """Raised when caller provides invalid input parameters (e.g. empty symbol)."""

    def __init__(self, message: str, *, param_name: Optional[str] = None) -> None:
        super().__init__(message)
        self.param_name = param_name


class AuthenticationError(PiaError):
    """Raised on HTTP 401 Unauthorized (invalid, missing, or revoked API key)."""

    def __init__(
        self,
        message: str = "Authentication failed. Verify that your API key is valid.",
        **kwargs: Any,
    ) -> None:
        super().__init__(message, **kwargs)


class PermissionError(PiaError):
    """Raised on HTTP 403 Forbidden (API key lacks required scope/tier)."""

    def __init__(
        self,
        message: str,
        *,
        required_scope: Optional[str] = None,
        **kwargs: Any,
    ) -> None:
        super().__init__(message, **kwargs)
        self.required_scope = required_scope


class RateLimitError(PiaError):
    """Raised on HTTP 429 Too Many Requests (per-minute or daily quota exceeded)."""

    def __init__(
        self,
        message: str,
        *,
        retry_after_seconds: Optional[int] = None,
        daily_limit: Optional[int] = None,
        daily_remaining: Optional[int] = None,
        minute_limit: Optional[int] = None,
        minute_remaining: Optional[int] = None,
        **kwargs: Any,
    ) -> None:
        super().__init__(message, **kwargs)
        self.retry_after_seconds = retry_after_seconds
        self.daily_limit = daily_limit
        self.daily_remaining = daily_remaining
        self.minute_limit = minute_limit
        self.minute_remaining = minute_remaining


class TimeoutError(PiaError):
    """Raised when an HTTP or WebSocket operation exceeds the configured timeout."""

    def __init__(
        self,
        message: str,
        *,
        timeout_seconds: float,
        **kwargs: Any,
    ) -> None:
        super().__init__(message, **kwargs)
        self.timeout_seconds = timeout_seconds


class NetworkError(PiaError):
    """Raised when network transport fails (connection refused, DNS error, socket drop)."""

    def __init__(
        self,
        message: str,
        *,
        cause: Optional[Exception] = None,
        **kwargs: Any,
    ) -> None:
        super().__init__(message, **kwargs)
        self.cause = cause


class ParseError(PiaError):
    """Raised when the server response cannot be decoded as valid JSON."""

    def __init__(
        self,
        message: str,
        *,
        raw_text: Optional[str] = None,
        **kwargs: Any,
    ) -> None:
        super().__init__(message, **kwargs)
        self.raw_text = raw_text


class ApiError(PiaError):
    """Raised on unexpected HTTP 4xx or 5xx server errors."""
