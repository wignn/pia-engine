-- =============================================================================
-- ATLSD Core — Consolidated Initial Migration
-- =============================================================================

-- ---------------------------------------------------------------------------
-- Forex News
-- ---------------------------------------------------------------------------

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


CREATE TABLE IF NOT EXISTS users (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email           TEXT NOT NULL UNIQUE,
    name            TEXT NOT NULL DEFAULT '',
    plan            TEXT NOT NULL DEFAULT 'free',
    is_active       BOOLEAN NOT NULL DEFAULT TRUE,
    email_verified  BOOLEAN NOT NULL DEFAULT FALSE,
    verify_token    TEXT,
    password_hash   TEXT,
    avatar_url      TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_plan ON users(plan);

CREATE TABLE IF NOT EXISTS oauth_accounts (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider        TEXT NOT NULL,
    provider_id     TEXT NOT NULL,
    provider_email  TEXT,
    access_token    TEXT,
    refresh_token   TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(provider, provider_id)
);

CREATE INDEX IF NOT EXISTS idx_oauth_provider ON oauth_accounts(provider, provider_id);
CREATE INDEX IF NOT EXISTS idx_oauth_user ON oauth_accounts(user_id);


CREATE TABLE IF NOT EXISTS api_keys (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    key_hash        TEXT NOT NULL UNIQUE,
    key_prefix      TEXT NOT NULL,
    label           TEXT NOT NULL DEFAULT 'default',
    permissions     TEXT[] NOT NULL DEFAULT '{}',
    is_active       BOOLEAN NOT NULL DEFAULT TRUE,
    last_used_at    TIMESTAMPTZ,
    expires_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_api_keys_hash ON api_keys(key_hash);
CREATE INDEX IF NOT EXISTS idx_api_keys_user ON api_keys(user_id);

CREATE TABLE IF NOT EXISTS tenant_configs (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    config_key      TEXT NOT NULL,
    config_value    JSONB NOT NULL DEFAULT '{}',
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, config_key)
);

CREATE INDEX IF NOT EXISTS idx_tenant_configs_user ON tenant_configs(user_id);
CREATE INDEX IF NOT EXISTS idx_tenant_configs_key ON tenant_configs(config_key);

CREATE TABLE IF NOT EXISTS usage_logs (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    api_key_id      UUID REFERENCES api_keys(id) ON DELETE SET NULL,
    endpoint        TEXT NOT NULL,
    method          TEXT NOT NULL,
    status_code     INT NOT NULL DEFAULT 200,
    response_ms     INT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_usage_user_date ON usage_logs(user_id, created_at DESC);


CREATE TABLE IF NOT EXISTS plans (
    id                  TEXT PRIMARY KEY,
    name                TEXT NOT NULL,
    price_idr           BIGINT NOT NULL DEFAULT 0,
    requests_per_day    INT NOT NULL DEFAULT 100,
    ws_connections      INT NOT NULL DEFAULT 1,
    x_usernames_max     INT NOT NULL DEFAULT 1,
    tv_symbols_max      INT NOT NULL DEFAULT 3,
    news_history_days   INT NOT NULL DEFAULT 1,
    rate_limit_per_min  INT NOT NULL DEFAULT 10,
    can_scrape          BOOLEAN NOT NULL DEFAULT FALSE,
    can_custom_rss      BOOLEAN NOT NULL DEFAULT FALSE,
    is_active           BOOLEAN NOT NULL DEFAULT TRUE,
    sort_order          INT NOT NULL DEFAULT 0
);

INSERT INTO plans (id, name, price_idr, requests_per_day, ws_connections, x_usernames_max, tv_symbols_max, news_history_days, rate_limit_per_min, can_scrape, can_custom_rss, sort_order)
VALUES
    ('free',       'Free',       0,       100,    1,   1,   3,   1,   10,   FALSE, FALSE, 0),
    ('starter',    'Starter',    149000,  5000,   3,   5,   10,  7,   60,   TRUE,  FALSE, 1),
    ('pro',        'Pro',        499000,  50000,  10,  20,  50,  30,  300,  TRUE,  TRUE,  2),
    ('enterprise', 'Enterprise', 0,       999999, 100, 100, 200, 365, 1000, TRUE,  TRUE,  3)
ON CONFLICT (id) DO NOTHING;


CREATE TABLE IF NOT EXISTS ohlcv_candles (
    symbol      TEXT NOT NULL,
    resolution  TEXT NOT NULL,
    time        TIMESTAMPTZ NOT NULL,
    open        DOUBLE PRECISION NOT NULL,
    high        DOUBLE PRECISION NOT NULL,
    low         DOUBLE PRECISION NOT NULL,
    close       DOUBLE PRECISION NOT NULL,
    volume      DOUBLE PRECISION NOT NULL DEFAULT 0,
    PRIMARY KEY (symbol, resolution, time)
);

CREATE INDEX IF NOT EXISTS idx_ohlcv_candles_time ON ohlcv_candles(time DESC);

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'timescaledb') THEN
        PERFORM create_hypertable('ohlcv_candles', 'time', if_not_exists => TRUE);
    END IF;
EXCEPTION
    WHEN OTHERS THEN
        NULL;
END $$;
