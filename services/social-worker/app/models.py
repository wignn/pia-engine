from __future__ import annotations

from dataclasses import dataclass, field

from .config import SourceAccount


@dataclass(frozen=True)
class TweetRecord:
    post_id: str
    platform: str
    source_account: str
    author_username: str
    author_display_name: str
    text: str
    url: str
    created_at: str
    fetched_at: str
    reply_count: int = 0
    retweet_count: int = 0
    like_count: int = 0
    quote_count: int = 0
    language: str = ""
    media_urls: tuple[str, ...] = field(default_factory=tuple)

    @property
    def event_id(self) -> str:
        return f"{self.platform}:{self.post_id}"

    def as_dict(self) -> dict:
        return {
            "event_id": self.event_id,
            "post_id": self.post_id,
            "platform": self.platform,
            "source_account": self.source_account,
            "author_username": self.author_username,
            "author_display_name": self.author_display_name,
            "text": self.text,
            "url": self.url,
            "created_at": self.created_at,
            "fetched_at": self.fetched_at,
            "reply_count": self.reply_count,
            "retweet_count": self.retweet_count,
            "like_count": self.like_count,
            "quote_count": self.quote_count,
            "language": self.language,
            "media_urls": list(self.media_urls),
        }


@dataclass
class AccountStatus:
    last_seen_post_id: str = ""
    last_success_at: str = ""
    last_error: str = ""
    fetched_count: int = 0


def account_key(account: SourceAccount) -> str:
    return account.key
