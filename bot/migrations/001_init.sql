-- Bot SQLite Schema
-- Lightweight, self-contained storage for Discord bot state

-- Moderation: warnings
CREATE TABLE IF NOT EXISTS mod_warnings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    guild_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    moderator_id INTEGER NOT NULL,
    reason TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_warnings_guild_user ON mod_warnings(guild_id, user_id);

-- Moderation: config (auto-role, log channel)
CREATE TABLE IF NOT EXISTS mod_config (
    guild_id INTEGER PRIMARY KEY,
    auto_role_id INTEGER,
    log_channel_id INTEGER
);

-- Forex news channel subscriptions
CREATE TABLE IF NOT EXISTS forex_channels (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    channel_id INTEGER NOT NULL,
    guild_id INTEGER NOT NULL UNIQUE,
    is_active INTEGER NOT NULL DEFAULT 1
);

-- Stock news channel subscriptions
CREATE TABLE IF NOT EXISTS stock_channels (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    channel_id INTEGER NOT NULL UNIQUE,
    guild_id INTEGER NOT NULL,
    tickers_filter TEXT,
    min_impact TEXT DEFAULT 'low',
    categories TEXT,
    mention_everyone INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1
);
CREATE INDEX IF NOT EXISTS idx_stock_channels_guild ON stock_channels(guild_id);

-- Calendar reminder channel subscriptions
CREATE TABLE IF NOT EXISTS calendar_channels (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    channel_id INTEGER NOT NULL,
    guild_id INTEGER NOT NULL UNIQUE,
    is_active INTEGER NOT NULL DEFAULT 1,
    mention_everyone INTEGER NOT NULL DEFAULT 0
);

-- Volatility spike channel subscriptions
CREATE TABLE IF NOT EXISTS volatility_channels (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    guild_id INTEGER NOT NULL UNIQUE,
    channel_id INTEGER NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1
);

-- X/Twitter feed channel subscriptions
CREATE TABLE IF NOT EXISTS twitter_channels (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    guild_id INTEGER NOT NULL UNIQUE,
    channel_id INTEGER NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1
);

-- Price alerts
CREATE TABLE IF NOT EXISTS price_alerts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    guild_id INTEGER NOT NULL,
    symbol TEXT NOT NULL,
    target_price REAL NOT NULL,
    direction TEXT NOT NULL,
    is_triggered INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    triggered_at TEXT
);
CREATE INDEX IF NOT EXISTS idx_price_alerts_active ON price_alerts(is_triggered, symbol);
CREATE INDEX IF NOT EXISTS idx_price_alerts_user ON price_alerts(user_id, is_triggered);

-- Unified dedup table: tracks what has been sent to prevent duplicates
CREATE TABLE IF NOT EXISTS sent_items (
    item_id TEXT PRIMARY KEY,
    item_type TEXT NOT NULL,
    source TEXT,
    sent_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_sent_items_type ON sent_items(item_type);
CREATE INDEX IF NOT EXISTS idx_sent_items_sent_at ON sent_items(sent_at);
