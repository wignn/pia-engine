CREATE TABLE IF NOT EXISTS options_snapshots (
    id TEXT PRIMARY KEY,
    symbol TEXT NOT NULL,
    underlying_price DOUBLE PRECISION NOT NULL,
    put_call_ratio DOUBLE PRECISION NOT NULL,
    max_pain_strike DOUBLE PRECISION NOT NULL,
    total_open_interest BIGINT NOT NULL,
    total_volume BIGINT NOT NULL,
    total_gex DOUBLE PRECISION NOT NULL,
    iv_atm DOUBLE PRECISION,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS options_contracts (
    contract_symbol TEXT PRIMARY KEY,
    symbol TEXT NOT NULL,
    option_type TEXT NOT NULL,
    strike DOUBLE PRECISION NOT NULL,
    expiration_date DATE NOT NULL,
    mark_price DOUBLE PRECISION NOT NULL,
    bid DOUBLE PRECISION,
    ask DOUBLE PRECISION,
    implied_volatility DOUBLE PRECISION NOT NULL,
    delta DOUBLE PRECISION NOT NULL,
    gamma DOUBLE PRECISION NOT NULL,
    theta DOUBLE PRECISION NOT NULL,
    vega DOUBLE PRECISION NOT NULL,
    gex DOUBLE PRECISION NOT NULL,
    open_interest BIGINT NOT NULL,
    volume BIGINT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_options_contracts_symbol_exp
    ON options_contracts (symbol, expiration_date, strike);
