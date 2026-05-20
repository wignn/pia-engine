CREATE TABLE IF NOT EXISTS users (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email           TEXT NOT NULL UNIQUE,
    name            TEXT NOT NULL DEFAULT '',
    plan            TEXT NOT NULL DEFAULT 'free',
    is_active       BOOLEAN NOT NULL DEFAULT TRUE,
    email_verified  BOOLEAN NOT NULL DEFAULT FALSE,
    verify_token    TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_plan ON users(plan);

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
