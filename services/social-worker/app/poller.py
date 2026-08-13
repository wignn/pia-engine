from __future__ import annotations

import asyncio
import contextlib
import logging
from typing import AsyncIterator, Awaitable, Callable

from .config import Config, SourceAccount
from .models import AccountStatus, TweetRecord
from .normalizer import iso
from .sources.base import SourceAdapter
from .sources.truth import TruthSocialSource
from .sources.twitter import TwitterSource

log = logging.getLogger("social_worker")
Fetcher = Callable[[SourceAccount, int], Awaitable[list[TweetRecord]]]


class PollingWorker:
    """Platform-agnostic scheduler, deduplicator, and event fan-out."""

    def __init__(
        self,
        config: Config,
        fetcher: Fetcher | None = None,
        sources: dict[str, SourceAdapter] | None = None,
    ):
        self.config = config
        self._fetcher = fetcher
        self._sources = sources or {
            "twitter": TwitterSource(
                config.db_path,
                auth_token=config.x_auth_token,
                ct0=config.x_ct0,
                account_name=config.x_account_name,
            ),
            "truth": TruthSocialSource(),
        }
        self._seen_ids: dict[str, set[str]] = {}
        self._last_seen: dict[str, str] = {}
        self._max_seen_ids = 5000
        self._status = {account.key: AccountStatus() for account in config.accounts}
        self._subscribers: set[asyncio.Queue[TweetRecord | None]] = set()
        self._stop = asyncio.Event()
        self._task: asyncio.Task | None = None

    @staticmethod
    def _record_sort_key(record: TweetRecord) -> tuple[str, str]:
        return record.created_at, record.post_id

    def _new_records(self, key: str, records: list[TweetRecord]) -> list[TweetRecord]:
        seen = self._seen_ids.setdefault(key, set())
        new_records = [record for record in records if record.event_id not in seen]
        seen.update(record.event_id for record in records)
        if len(seen) > self._max_seen_ids:
            self._seen_ids[key] = {record.event_id for record in records}
        return new_records

    def _update_last_seen(self, key: str, records: list[TweetRecord]) -> None:
        if records:
            self._last_seen[key] = records[-1].post_id

    async def start(self) -> None:
        if self._task is None:
            self._stop.clear()
            self._task = asyncio.create_task(self.run(), name="social-polling-worker")

    async def stop(self) -> None:
        self._stop.set()
        if self._task:
            await self._task
            self._task = None
        for source in self._sources.values():
            await source.close()
        for queue in tuple(self._subscribers):
            self._offer(queue, None)

    async def run(self) -> None:
        while not self._stop.is_set():
            await asyncio.gather(*(self.poll_account(account) for account in self.config.accounts))
            try:
                await asyncio.wait_for(self._stop.wait(), timeout=self.config.poll_seconds)
            except asyncio.TimeoutError:
                pass

    async def poll_account(self, account: SourceAccount) -> None:
        key = account.key
        try:
            records = sorted(await self._fetch(account), key=self._record_sort_key)
            new_records = self._new_records(key, records)
            self._update_last_seen(key, records)
            status = self._status[key]
            status.last_seen_post_id = self._last_seen.get(key, "")
            status.last_success_at = iso(None)
            status.last_error = ""
            status.fetched_count += len(records)
            for record in new_records:
                for queue in tuple(self._subscribers):
                    self._offer(queue, record)
        except Exception as exc:
            self._status[key].last_error = f"{type(exc).__name__}: {exc}"
            log.warning("poll failed for %s: %s", key, exc)

    async def _fetch(self, account: SourceAccount) -> list[TweetRecord]:
        if self._fetcher is not None:
            return await self._fetcher(account, self.config.tweet_limit)
        source = self._sources.get(account.platform)
        if source is None:
            raise ValueError(f"platform tidak didukung: {account.platform}")
        return await source.fetch_latest(account, self.config.tweet_limit)

    def _offer(self, queue: asyncio.Queue[TweetRecord | None], item: TweetRecord | None) -> None:
        try:
            queue.put_nowait(item)
        except asyncio.QueueFull:
            with contextlib.suppress(asyncio.QueueEmpty):
                queue.get_nowait()
            queue.put_nowait(item)

    def subscribe(self) -> AsyncIterator[TweetRecord]:
        queue: asyncio.Queue[TweetRecord | None] = asyncio.Queue(maxsize=1000)
        self._subscribers.add(queue)

        async def stream() -> AsyncIterator[TweetRecord]:
            try:
                while True:
                    item = await queue.get()
                    if item is None:
                        return
                    yield item
            finally:
                self._subscribers.discard(queue)

        return stream()

    def status(self) -> dict[str, AccountStatus]:
        return {key: AccountStatus(**vars(value)) for key, value in self._status.items()}
