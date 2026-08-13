from __future__ import annotations

import asyncio
from datetime import datetime, timezone

from twscrape import API

from ..config import SourceAccount
from ..models import TweetRecord
from ..normalizer import normalize_twitter_tweet


class TwitterSource:
    def __init__(
        self,
        db_path: str,
        auth_token: str = "",
        ct0: str = "",
        account_name: str = "worker",
    ):
        self.api = API(db_path)
        self._auth_token = auth_token
        self._ct0 = ct0
        self._account_name = account_name
        self._cookies_loaded = False
        self._cookies_lock = asyncio.Lock()
        self._user_ids: dict[str, int] = {}

    async def _ensure_cookies(self) -> None:
        if self._cookies_loaded:
            return
        async with self._cookies_lock:
            if self._cookies_loaded:
                return
            if self._auth_token and self._ct0:
                await self.api.pool.add_account_cookies(
                    self._account_name,
                    f"auth_token={self._auth_token}; ct0={self._ct0}",
                )
            self._cookies_loaded = True

    async def fetch_latest(self, account: SourceAccount, limit: int) -> list[TweetRecord]:
        await self._ensure_cookies()
        user_id = self._user_ids.get(account.username)
        if user_id is None:
            user = await self.api.user_by_login(account.username)
            if user is None:
                raise ValueError(f"akun Twitter tidak ditemukan: @{account.username}")
            user_id = int(user.id)
            self._user_ids[account.username] = user_id
        fetched_at = datetime.now(timezone.utc)
        return [
            normalize_twitter_tweet(tweet, account.username, fetched_at)
            async for tweet in self.api.user_tweets(user_id, limit=limit)
        ]

    async def close(self) -> None:
        return None
