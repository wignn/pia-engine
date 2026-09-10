"""Official PIA SDK - Logging & Security Sanitization."""

from __future__ import annotations

import logging
from typing import Any

from .errors import redact_sensitive

SDK_LOGGER_NAME = "pia.sdk"


class RedactingFormatter(logging.Formatter):
    """Custom logging formatter that automatically redacts API keys and tokens."""

    def format(self, record: logging.LogRecord) -> str:
        original = super().format(record)
        return redact_sensitive(original)


def setup_logger(debug: bool = False) -> logging.Logger:
    """Configures and returns the official SDK logger with sensitive data redaction."""
    logger = logging.getLogger(SDK_LOGGER_NAME)

    if not logger.handlers:
        handler = logging.StreamHandler()
        formatter = RedactingFormatter(
            fmt="[%(asctime)s] [%(name)s] [%(levelname)s] %(message)s",
            datefmt="%Y-%m-%d %H:%M:%S",
        )
        handler.setFormatter(formatter)
        logger.addHandler(handler)

    logger.setLevel(logging.DEBUG if debug else logging.WARNING)
    return logger
