CREATE TABLE IF NOT EXISTS calendar_channels (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    guild_id INTEGER NOT NULL UNIQUE,
    channel_id INTEGER NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    mention_everyone INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS forex_channels (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    guild_id INTEGER NOT NULL UNIQUE,
    channel_id INTEGER NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS stock_channels (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    channel_id INTEGER NOT NULL UNIQUE,
    guild_id INTEGER NOT NULL,
    tickers_filter TEXT,
    min_impact TEXT,
    categories TEXT,
    mention_everyone INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS twitter_channels (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    guild_id INTEGER NOT NULL UNIQUE,
    channel_id INTEGER NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS volatility_channels (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    guild_id INTEGER NOT NULL UNIQUE,
    channel_id INTEGER NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS sent_items (
    item_id TEXT PRIMARY KEY,
    item_type TEXT NOT NULL,
    source TEXT NOT NULL,
    sent_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS mod_warnings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    guild_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    moderator_id INTEGER NOT NULL,
    reason TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS mod_config (
    guild_id INTEGER PRIMARY KEY,
    auto_role_id INTEGER,
    log_channel_id INTEGER,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS price_alerts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    guild_id INTEGER NOT NULL,
    symbol TEXT NOT NULL,
    target_price REAL NOT NULL,
    direction TEXT NOT NULL,
    is_triggered INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    triggered_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_calendar_channels_active ON calendar_channels(is_active);
CREATE INDEX IF NOT EXISTS idx_forex_channels_active ON forex_channels(is_active);
CREATE INDEX IF NOT EXISTS idx_stock_channels_active ON stock_channels(is_active);
CREATE INDEX IF NOT EXISTS idx_twitter_channels_active ON twitter_channels(is_active);
CREATE INDEX IF NOT EXISTS idx_volatility_channels_active ON volatility_channels(is_active);
CREATE INDEX IF NOT EXISTS idx_sent_items_type_sent_at ON sent_items(item_type, sent_at);
CREATE INDEX IF NOT EXISTS idx_mod_warnings_user ON mod_warnings(guild_id, user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_price_alerts_user_active ON price_alerts(user_id, is_triggered, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_price_alerts_symbol_active ON price_alerts(symbol, is_triggered);
