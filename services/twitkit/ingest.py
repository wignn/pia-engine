"""Production Twikit adapter: X session -> normalized events -> NATS."""
from __future__ import annotations

import asyncio
import json
import logging
import os
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from twikit import Client
from twikit.errors import TooManyRequests

LOG = logging.getLogger("twitkit")
logging.basicConfig(level=os.getenv("LOG_LEVEL", "INFO"), format="%(asctime)s %(levelname)s %(message)s")
TARGETS = tuple(x.strip().lstrip("@").lower() for x in os.getenv("SOCIAL_ACCOUNTS", "macropaperr,BullTheoryio,WhaleInsider,coinbureau").split(",") if x.strip())
COOKIES_FILE = Path(os.getenv("TWIKIT_COOKIES_FILE", "cookies.json"))
NATS_URL = os.getenv("NATS_URL", "")
NATS_SUBJECT = os.getenv("NATS_SUBJECT", "social.posts")
TWEET_LIMIT = int(os.getenv("TWITTER_TWEET_LIMIT", "20"))
POLL_SECONDS = int(os.getenv("TWITTER_POLL_SECONDS", "900"))
TARGET_DELAY = int(os.getenv("TWIKIT_TARGET_DELAY", "30"))


def _iso(value: Any) -> str:
    return value.astimezone(timezone.utc).isoformat() if isinstance(value, datetime) else str(value)


def normalize(tweet: Any, username: str, fetched_at: str | None = None) -> dict[str, Any]:
    author = getattr(tweet, "user", None)
    handle = getattr(author, "screen_name", None) or username
    return {"event_id": f"twitter:{tweet.id}", "post_id": str(tweet.id), "platform": "twitter", "source_account": username,
            "author_username": handle, "author_display_name": getattr(author, "name", None) or handle,
            "text": tweet.text, "url": f"https://x.com/{handle}/status/{tweet.id}",
            "created_at": _iso(tweet.created_at_datetime), "fetched_at": fetched_at or datetime.now(timezone.utc).isoformat(),
            "reply_count": int(getattr(tweet, "reply_count", 0) or 0), "retweet_count": int(getattr(tweet, "retweet_count", 0) or 0),
            "like_count": int(getattr(tweet, "favorite_count", 0) or 0), "quote_count": int(getattr(tweet, "quote_count", 0) or 0),
            "language": getattr(tweet, "lang", "") or "", "media_urls": []}


async def fetch_once(client: Client, seen: set[str]) -> list[dict[str, Any]]:
    events = []
    for username in TARGETS:
        try:
            user = await client.get_user_by_screen_name(username)
            tweets = await user.get_tweets("Tweets", count=TWEET_LIMIT)
            for tweet in tweets:
                event = normalize(tweet, username)
                if event["event_id"] not in seen:
                    seen.add(event["event_id"])
                    events.append(event)
            LOG.info("fetched target=%s count=%d", username, len(tweets))
        except TooManyRequests as exc:
            LOG.warning("rate limited target=%s retry_after=%s", username, getattr(exc, "headers", {}).get("retry-after", "unknown"))
        except Exception:
            LOG.exception("target fetch failed target=%s", username)
        await asyncio.sleep(TARGET_DELAY)
    return events


async def publish(events: list[dict[str, Any]]) -> None:
    if not events or not NATS_URL:
        return
    import nats
    nc = await nats.connect(servers=[NATS_URL])
    try:
        for event in events:
            await nc.publish(NATS_SUBJECT, json.dumps(event, ensure_ascii=False).encode())
        await nc.flush()
        LOG.info("published subject=%s count=%d", NATS_SUBJECT, len(events))
    finally:
        await nc.drain()


async def run() -> None:
    if not COOKIES_FILE.exists():
        raise RuntimeError(f"cookie file tidak ditemukan: {COOKIES_FILE}")
    client = Client("en-US")
    client.load_cookies(str(COOKIES_FILE))
    seen: set[str] = set()
    while True:
        events = await fetch_once(client, seen)
        await publish(events)
        LOG.info("poll complete new_events=%d next_poll_seconds=%d", len(events), POLL_SECONDS)
        await asyncio.sleep(POLL_SECONDS)


if __name__ == "__main__":
    asyncio.run(run())
