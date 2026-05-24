ALTER TABLE forex_news_sources
    ADD COLUMN IF NOT EXISTS category TEXT NOT NULL DEFAULT 'forex',
    ADD COLUMN IF NOT EXISTS poll_interval_sec INTEGER NOT NULL DEFAULT 45,
    ADD COLUMN IF NOT EXISTS priority INTEGER NOT NULL DEFAULT 100,
    ADD COLUMN IF NOT EXISTS etag TEXT,
    ADD COLUMN IF NOT EXISTS last_modified TEXT,
    ADD COLUMN IF NOT EXISTS last_success_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS last_error_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS blocked_until TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS next_allowed_poll_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS consecutive_403 INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS success_count BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS error_count BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS forbidden_count BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS parse_error_count BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS last_status INTEGER,
    ADD COLUMN IF NOT EXISTS last_latency_ms BIGINT,
    ADD COLUMN IF NOT EXISTS last_error_message TEXT,
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

CREATE INDEX IF NOT EXISTS idx_forex_news_sources_active_priority
    ON forex_news_sources(is_active, source_type, priority);

CREATE INDEX IF NOT EXISTS idx_forex_news_sources_next_poll
    ON forex_news_sources(next_allowed_poll_at);

CREATE UNIQUE INDEX IF NOT EXISTS idx_forex_news_sources_rss_url_unique
    ON forex_news_sources(rss_url)
    WHERE rss_url IS NOT NULL;

INSERT INTO forex_news_sources (id, name, slug, source_type, url, rss_url, category, poll_interval_sec, priority, is_active, updated_at)
VALUES
    ('feed-investinglive', 'InvestingLive', 'investinglive', 'rss', 'https://investinglive.com', 'https://investinglive.com/feed/news/', 'forex', 45, 10, TRUE, NOW()),
    ('feed-fxstreet', 'FXStreet', 'fxstreet', 'rss', 'https://www.fxstreet.com', 'https://www.fxstreet.com/rss/news', 'forex', 45, 20, TRUE, NOW()),
    ('feed-marketpulse', 'MarketPulse', 'marketpulse', 'rss', 'https://www.marketpulse.com', 'https://www.marketpulse.com/feed/', 'macro', 300, 30, TRUE, NOW()),
    ('feed-actionforex', 'ActionForex', 'actionforex', 'rss', 'https://www.actionforex.com', 'https://www.actionforex.com/feed/', 'forex', 45, 40, TRUE, NOW()),
    ('feed-investing-com-forex-news', 'Investing.com - Forex News', 'investing-com-forex-news', 'rss', 'https://id.investing.com/news/forex-news', 'https://id.investing.com/rss/news_301.rss', 'forex', 45, 50, TRUE, NOW()),
    ('feed-investing-com-economic-indicators', 'Investing.com - Economic Indicators', 'investing-com-economic-indicators', 'rss', 'https://id.investing.com/news/economic-indicators', 'https://id.investing.com/rss/news_95.rss', 'economic', 120, 60, TRUE, NOW()),
    ('feed-federal-reserve', 'Federal Reserve', 'federal-reserve', 'rss', 'https://www.federalreserve.gov', 'https://www.federalreserve.gov/feeds/press_all.xml', 'central_bank', 600, 70, TRUE, NOW()),
    ('feed-ecb', 'ECB', 'ecb', 'rss', 'https://www.ecb.europa.eu', 'https://www.ecb.europa.eu/rss/press.html', 'central_bank', 600, 80, TRUE, NOW()),
    ('feed-bank-of-england', 'Bank of England', 'bank-of-england', 'rss', 'https://www.bankofengland.co.uk', 'https://www.bankofengland.co.uk/rss/news', 'central_bank', 600, 90, TRUE, NOW()),
    ('feed-bank-of-canada', 'Bank of Canada', 'bank-of-canada', 'rss', 'https://www.bankofcanada.ca', 'https://www.bankofcanada.ca/content_type/press-releases/feed/', 'central_bank', 600, 100, TRUE, NOW())
ON CONFLICT (slug) DO UPDATE SET
    name = EXCLUDED.name,
    source_type = EXCLUDED.source_type,
    url = EXCLUDED.url,
    rss_url = EXCLUDED.rss_url,
    category = EXCLUDED.category,
    poll_interval_sec = EXCLUDED.poll_interval_sec,
    priority = EXCLUDED.priority,
    updated_at = NOW();
