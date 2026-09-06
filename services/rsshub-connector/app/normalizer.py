from __future__ import annotations

from datetime import datetime, timezone

from .models import TweetRecord


def iso(value: datetime | None) -> str:
    value = value or datetime.now(timezone.utc)
    if value.tzinfo is None:
        value = value.replace(tzinfo=timezone.utc)
    return value.astimezone(timezone.utc).isoformat().replace("+00:00", "Z")


def normalize_twitter_tweet(tweet, source_account: str, fetched_at: datetime | None = None) -> TweetRecord:
    """Convert a twscrape Tweet into the cross-platform event schema."""
    user = tweet.user
    media = getattr(tweet, "media", None)
    media_urls = tuple(
        [photo.url for photo in getattr(media, "photos", [])]
        + [video.thumbnailUrl for video in getattr(media, "videos", [])]
        + [gif.thumbnailUrl for gif in getattr(media, "animated", [])]
    )
    return TweetRecord(
        post_id=str(tweet.id),
        platform="twitter",
        source_account=source_account,
        author_username=str(getattr(user, "username", "")),
        author_display_name=str(getattr(user, "displayname", "") or getattr(user, "displayName", "")),
        text=str(getattr(tweet, "rawContent", "")),
        url=str(getattr(tweet, "url", "")),
        created_at=iso(getattr(tweet, "date", None)),
        fetched_at=iso(fetched_at),
        reply_count=int(getattr(tweet, "replyCount", 0) or 0),
        retweet_count=int(getattr(tweet, "retweetCount", 0) or 0),
        like_count=int(getattr(tweet, "likeCount", 0) or 0),
        quote_count=int(getattr(tweet, "quoteCount", 0) or 0),
        language=str(getattr(tweet, "lang", "") or ""),
        media_urls=media_urls,
    )
