from __future__ import annotations

from ..config import SourceAccount
from ..models import TweetRecord


class TruthSocialSource:
    """Truth Social adapter boundary.

    Truth Social access is deliberately explicit: configure an approved/public
    client in this class rather than reusing Twitter cookies or endpoints.
    """

    async def fetch_latest(self, account: SourceAccount, limit: int) -> list[TweetRecord]:
        raise RuntimeError(
            "Truth Social source belum dikonfigurasi. Tambahkan client/API resmi "
            "yang diizinkan pada app/sources/truth.py."
        )

    async def close(self) -> None:
        return None
