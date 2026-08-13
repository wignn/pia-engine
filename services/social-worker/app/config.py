from __future__ import annotations

import os
from dataclasses import dataclass, field


SUPPORTED_PLATFORMS = frozenset({"twitter", "truth"})


def _positive_float(name: str, default: float) -> float:
    raw = os.getenv(name, str(default))
    try:
        value = float(raw)
    except ValueError as exc:
        raise ValueError(f"{name} harus berupa angka") from exc
    if value <= 0:
        raise ValueError(f"{name} harus lebih besar dari 0")
    return value


def _positive_int(name: str, default: int) -> int:
    raw = os.getenv(name, str(default))
    try:
        value = int(raw)
    except ValueError as exc:
        raise ValueError(f"{name} harus berupa bilangan bulat") from exc
    if value <= 0:
        raise ValueError(f"{name} harus lebih besar dari 0")
    return value


@dataclass(frozen=True, order=True)
class SourceAccount:
    platform: str
    username: str

    @property
    def key(self) -> str:
        return f"{self.platform}:{self.username}"


def parse_accounts(raw: str) -> tuple[SourceAccount, ...]:
    accounts: list[SourceAccount] = []
    seen: set[SourceAccount] = set()
    for item in raw.split(","):
        item = item.strip()
        if not item:
            continue
        try:
            platform, username = item.split(":", 1)
        except ValueError as exc:
            raise ValueError(f"akun tidak valid: {item!r}; gunakan platform:username") from exc
        platform = platform.strip().lower()
        username = username.strip().lstrip("@").lower()
        if platform not in SUPPORTED_PLATFORMS:
            raise ValueError(f"platform tidak didukung: {platform!r}")
        if not username:
            raise ValueError(f"username kosong untuk platform: {platform}")
        account = SourceAccount(platform, username)
        if account not in seen:
            accounts.append(account)
            seen.add(account)
    if not accounts:
        raise ValueError("SOCIAL_ACCOUNTS harus berisi minimal satu platform:username")
    return tuple(accounts)


@dataclass(frozen=True)
class Config:
    accounts: tuple[SourceAccount, ...]
    poll_seconds: float = 60.0
    tweet_limit: int = 20
    db_path: str = "accounts.db"
    nats_url: str = "nats://127.0.0.1:4222"
    nats_subject: str = "social.posts"
    nats_creds: str = ""
    x_auth_token: str = field(default="", repr=False)
    x_ct0: str = field(default="", repr=False)
    x_account_name: str = "worker"

    @classmethod
    def from_env(cls) -> "Config":
        raw_accounts = os.getenv("SOCIAL_ACCOUNTS", "")
        if not raw_accounts:
            # Backward-compatible input; new deployments should use SOCIAL_ACCOUNTS.
            raw_accounts = ",".join(f"twitter:{x}" for x in os.getenv("TWITTER_ACCOUNTS", "").split(",") if x.strip())
        x_auth_token = os.getenv("X_AUTH_TOKEN", "").strip()
        x_ct0 = os.getenv("X_CT0", "").strip()
        if bool(x_auth_token) != bool(x_ct0):
            raise ValueError("X_AUTH_TOKEN dan X_CT0 harus diisi bersama")

        return cls(
            accounts=parse_accounts(raw_accounts),
            poll_seconds=_positive_float("TWITTER_POLL_SECONDS", 60.0),
            tweet_limit=_positive_int("TWITTER_TWEET_LIMIT", 20),
            db_path=os.getenv("TWITTER_DB", "accounts.db"),
            nats_url=os.getenv("NATS_URL", "nats://127.0.0.1:4222"),
            nats_subject=os.getenv("NATS_SUBJECT", "social.posts"),
            nats_creds=os.getenv("NATS_CREDS", ""),
            x_auth_token=x_auth_token,
            x_ct0=x_ct0,
            x_account_name=os.getenv("X_ACCOUNT_NAME", "worker").strip() or "worker",
        )
