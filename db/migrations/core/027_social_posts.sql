CREATE TABLE IF NOT EXISTS news.social_posts (
    event_id TEXT PRIMARY KEY,
    post_id TEXT NOT NULL,
    platform TEXT NOT NULL,
    source_account TEXT NOT NULL,
    author_username TEXT NOT NULL,
    author_display_name TEXT NOT NULL,
    text TEXT NOT NULL,
    url TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    fetched_at TIMESTAMPTZ NOT NULL,
    reply_count BIGINT NOT NULL DEFAULT 0,
    retweet_count BIGINT NOT NULL DEFAULT 0,
    like_count BIGINT NOT NULL DEFAULT 0,
    quote_count BIGINT NOT NULL DEFAULT 0,
    language TEXT NOT NULL DEFAULT '',
    media_urls JSONB NOT NULL DEFAULT '[]'::jsonb,
    inserted_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_social_posts_created_at
    ON news.social_posts (created_at DESC, event_id DESC);

CREATE INDEX IF NOT EXISTS idx_social_posts_platform_account_created
    ON news.social_posts (platform, source_account, created_at DESC, event_id DESC);
