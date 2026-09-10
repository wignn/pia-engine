"""Official PIA SDK - Configuration & Validation."""

from __future__ import annotations

import os
from dataclasses import dataclass, field
from typing import Any, Dict, Optional
from urllib.parse import urlparse

from .errors import ConfigurationError

DEFAULT_BASE_URL = "https://api-engine.wign.dev"
DEFAULT_WS_URL = "wss://api-engine.wign.dev/api/v1/ws"
DEFAULT_TIMEOUT_SECONDS = 15.0
DEFAULT_MAX_RETRIES = 3
DEFAULT_RETRY_DELAY_SECONDS = 0.5
DEFAULT_MAX_RETRY_DELAY_SECONDS = 10.0


def _resolve_env_key() -> Optional[str]:
    return (
        os.environ.get("PIA_API_KEY")
        or os.environ.get("ATSLD_API_KEY")
        or os.environ.get("CORE_API_KEY")
    )


@dataclass
class PiaConfig:
    api_key: str
    base_url: str = DEFAULT_BASE_URL
    ws_url: str = DEFAULT_WS_URL
    timeout: float = DEFAULT_TIMEOUT_SECONDS
    max_retries: int = DEFAULT_MAX_RETRIES
    retry_delay: float = DEFAULT_RETRY_DELAY_SECONDS
    max_retry_delay: float = DEFAULT_MAX_RETRY_DELAY_SECONDS
    headers: Dict[str, str] = field(default_factory=dict)
    debug: bool = False

    @classmethod
    def resolve(
        cls,
        api_key: Optional[str] = None,
        base_url: Optional[str] = None,
        ws_url: Optional[str] = None,
        timeout: Optional[float] = None,
        max_retries: Optional[int] = None,
        retry_delay: Optional[float] = None,
        max_retry_delay: Optional[float] = None,
        headers: Optional[Dict[str, str]] = None,
        debug: bool = False,
    ) -> PiaConfig:
        resolved_key = (api_key or _resolve_env_key() or "").strip()
        if not resolved_key:
            raise ConfigurationError(
                "API key is required. Pass api_key='...' or set the PIA_API_KEY environment variable."
            )

        raw_base = (base_url or DEFAULT_BASE_URL).strip().rstrip("/")
        parsed_base = urlparse(raw_base)
        if parsed_base.scheme not in ("http", "https") or not parsed_base.netloc:
            raise ConfigurationError(
                f"Invalid base_url '{raw_base}'. Must be a valid HTTP/HTTPS URL."
            )

        raw_ws = (ws_url or DEFAULT_WS_URL).strip().rstrip("/")
        parsed_ws = urlparse(raw_ws)
        if parsed_ws.scheme not in ("ws", "wss") or not parsed_ws.netloc:
            raise ConfigurationError(
                f"Invalid ws_url '{raw_ws}'. Must be a valid WS/WSS URL."
            )

        resolved_timeout = timeout if timeout is not None else DEFAULT_TIMEOUT_SECONDS
        if resolved_timeout < 0.1:
            raise ConfigurationError("timeout must be at least 0.1 seconds.")

        resolved_retries = max_retries if max_retries is not None else DEFAULT_MAX_RETRIES
        if resolved_retries < 0:
            raise ConfigurationError("max_retries cannot be negative.")

        return cls(
            api_key=resolved_key,
            base_url=raw_base,
            ws_url=raw_ws,
            timeout=resolved_timeout,
            max_retries=resolved_retries,
            retry_delay=retry_delay if retry_delay is not None else DEFAULT_RETRY_DELAY_SECONDS,
            max_retry_delay=max_retry_delay if max_retry_delay is not None else DEFAULT_MAX_RETRY_DELAY_SECONDS,
            headers=dict(headers or {}),
            debug=debug,
        )
