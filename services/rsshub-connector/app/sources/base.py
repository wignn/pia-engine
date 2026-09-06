from __future__ import annotations

from typing import Protocol

from ..config import SourceAccount
from ..models import TweetRecord


class SourceAdapter(Protocol):
    async def fetch_latest(self, account: SourceAccount, limit: int) -> list[TweetRecord]: ...

    async def close(self) -> None: ...
