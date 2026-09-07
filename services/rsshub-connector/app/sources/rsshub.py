from __future__ import annotations

import asyncio
import html
import re
import xml.etree.ElementTree as ET
from datetime import datetime, timezone
from email.utils import parsedate_to_datetime


_TAG_RE = re.compile(r"<[^>]+>")
_SRC_RE = re.compile(r"<img[^>]+src=[\\\"']([^\\\"']+)", re.IGNORECASE)


def clean_html_text(value: str) -> str:
    value = html.unescape(value or "")
    value = re.sub(r"<br\\s*/?>", "\\n", value, flags=re.IGNORECASE)
    value = re.sub(r"<script\\b[^>]*>.*?</script>|<style\\b[^>]*>.*?</style>", "", value, flags=re.IGNORECASE | re.DOTALL)
    value = _TAG_RE.sub("", value)
    return re.sub(r"[ \\t]+", " ", value).strip()


def html_image_urls(value: str) -> list[str]:
    return [html.unescape(url).strip() for url in _SRC_RE.findall(value or "") if url.startswith(("http://", "https://"))]


def media_url(element: ET.Element) -> str:
    return (element.attrib.get("url") or element.attrib.get("href") or "").strip()


def element_local_name(element: ET.Element) -> str:
    return element.tag.rsplit("}", 1)[-1].lower()


def is_image_url(url: str) -> bool:
    return bool(url) and url.startswith(("http://", "https://"))

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
            description = item.findtext('description') or ''
            encoded = next(
                (child.text or '' for child in item.iter() if element_local_name(child) == 'encoded'),
                '',
            )
            raw_text = description or item.findtext('title') or ''
            text = clean_html_text(raw_text)
            published = (item.findtext('pubDate') or now).strip()
            try:
                created_at = parsedate_to_datetime(published).astimezone(timezone.utc).isoformat().replace('+00:00', 'Z')
            except (TypeError, ValueError):
                created_at = now
            if not guid:
                continue
            media_urls = []
            for element in item.iter():
                tag = element.tag.rsplit('}', 1)[-1].lower()
                if tag not in {'content', 'thumbnail', 'enclosure'}:
                    continue
                media_url = (element.attrib.get('url') or element.attrib.get('href') or '').strip()
                media_type = (element.attrib.get('type') or '').lower()
                if media_url and (media_type.startswith('image/') or tag in {'content', 'thumbnail'}):
                    media_urls.append(media_url)
            media_urls.extend(html_image_urls(description))
            media_urls.extend(html_image_urls(encoded))
            media_urls = [url for url in media_urls if is_image_url(url)]
            media_urls = list(dict.fromkeys(media_urls))
            records.append(TweetRecord(
                post_id=guid.rsplit('/', 1)[-1], platform='twitter',
                source_account=account.username, author_username=account.username,
                author_display_name=account.username, text=text, url=link,
                created_at=created_at, fetched_at=now, media_urls=tuple(media_urls),
            ))

        return records

    async def close(self) -> None:
        await self.client.aclose()
