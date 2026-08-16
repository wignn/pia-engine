CREATE TABLE IF NOT EXISTS trading_halts (
    source TEXT NOT NULL,
    symbol TEXT NOT NULL,
    halt_date DATE NOT NULL,
    halt_time TIME,
    issue_name TEXT,
    market TEXT,
    reason_code TEXT,
    resume_date DATE,
    resume_quote_time TIME,
    resume_trade_time TIME,
    raw_json JSONB,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (source, symbol, halt_date, halt_time)
);

CREATE INDEX IF NOT EXISTS idx_trading_halts_symbol_date
    ON trading_halts (symbol, halt_date DESC);

CREATE TABLE IF NOT EXISTS corporate_actions (
    id TEXT PRIMARY KEY,
    source TEXT NOT NULL,
    symbol TEXT NOT NULL,
    action_type TEXT NOT NULL,
    ex_date DATE NOT NULL,
    amount DOUBLE PRECISION,
    ratio TEXT,
    description TEXT,
    raw_json JSONB,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_corporate_actions_symbol_date
    ON corporate_actions (symbol, ex_date DESC);

CREATE TABLE IF NOT EXISTS realized_volatility (
    symbol TEXT NOT NULL,
    window_days INTEGER NOT NULL,
    date DATE NOT NULL,
    realized_volatility DOUBLE PRECISION NOT NULL,
    source TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (symbol, window_days, date)
);

CREATE INDEX IF NOT EXISTS idx_realized_volatility_symbol_date
    ON realized_volatility (symbol, date DESC);
