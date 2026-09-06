from __future__ import annotations

import asyncio
import xml.etree.ElementTree as ET
from datetime import datetime, timezone
from email.utils import parsedate_to_datetime

import httpx

from ..models import TweetRecord
from ..config import SourceAccount


class RSSHubSource:
    def __init__(self, base_url: str):
        self.base_url = base_url.rstrip('/')
        self.client = httpx.AsyncClient(timeout=30, follow_redirects=True)

    async def fetch_latest(self, account: SourceAccount, limit: int) -> list[TweetRecord]:
        url = f"{self.base_url}/twitter/user/{account.username}"
        response = await self.client.get(url)
        response.raise_for_status()
        root = ET.fromstring(response.content)
        items = root.findall('.//item')[:limit]
        now = datetime.now(timezone.utc).isoformat().replace('+00:00', 'Z')
        records = []
        for item in items:
            guid = (item.findtext('guid') or item.findtext('link') or '').strip()
            link = (item.findtext('link') or '').strip()
            text = (item.findtext('description') or item.findtext('title') or '').strip()
            published = (item.findtext('pubDate') or now).strip()
            try:
                created_at = parsedate_to_datetime(published).astimezone(timezone.utc).isoformat().replace('+00:00', 'Z')
            except (TypeError, ValueError):
                created_at = now
            if not guid:
                continue
            records.append(TweetRecord(
                post_id=guid.rsplit('/', 1)[-1], platform='twitter',
                source_account=account.username, author_username=account.username,
                author_display_name=account.username, text=text, url=link,
                created_at=created_at, fetched_at=now,
            ))
        return records

    async def close(self) -> None:
        await self.client.aclose()
