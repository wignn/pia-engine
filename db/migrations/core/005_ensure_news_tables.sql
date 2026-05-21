-- Self-healing migration to ensure forex news tables exist

CREATE TABLE IF NOT EXISTS forex_news_sources (
    id         TEXT PRIMARY KEY,
    name       TEXT NOT NULL,
    slug       TEXT NOT NULL UNIQUE,
    source_type TEXT NOT NULL DEFAULT 'rss',
    url        TEXT NOT NULL,
    rss_url    TEXT,
    is_active  BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS forex_news_articles (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_id        TEXT REFERENCES forex_news_sources(id),
    content_hash     TEXT NOT NULL UNIQUE,
    original_url     TEXT NOT NULL,
    original_title   TEXT NOT NULL,
    original_content TEXT,
    translated_title TEXT DEFAULT '',
    summary          TEXT,
    is_processed     BOOLEAN NOT NULL DEFAULT FALSE,
    processed_at     TIMESTAMPTZ,
    published_at     TIMESTAMPTZ,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_forex_news_articles_processed_at ON forex_news_articles(processed_at DESC NULLS LAST);
CREATE INDEX IF NOT EXISTS idx_forex_news_articles_content_hash ON forex_news_articles(content_hash);

CREATE TABLE IF NOT EXISTS forex_news_analyses (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    article_id     UUID REFERENCES forex_news_articles(id) ON DELETE CASCADE,
    sentiment      TEXT,
    impact_level   TEXT,
    currency_pairs TEXT,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_forex_news_analyses_article_id ON forex_news_analyses(article_id);

CREATE TABLE IF NOT EXISTS stock_news (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    content_hash   TEXT NOT NULL UNIQUE,
    original_url   TEXT NOT NULL,
    title          TEXT NOT NULL,
    summary        TEXT,
    source_name    TEXT,
    category       TEXT,
    tickers        TEXT,
    sentiment      TEXT,
    impact_level   TEXT,
    is_processed   BOOLEAN NOT NULL DEFAULT FALSE,
    processed_at   TIMESTAMPTZ,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_stock_news_processed_at ON stock_news(processed_at DESC);
CREATE INDEX IF NOT EXISTS idx_stock_news_content_hash ON stock_news(content_hash);
