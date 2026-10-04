CREATE TABLE IF NOT EXISTS market.options_snapshot_history (
    snapshot_id BIGSERIAL PRIMARY KEY,
    snapshot_kind TEXT NOT NULL CHECK (snapshot_kind IN ('summary', 'chain')),
    symbol TEXT NOT NULL,
    observed_at TIMESTAMPTZ NOT NULL,
    source_observed_at TIMESTAMPTZ,
    underlying_price DOUBLE PRECISION,
    put_call_ratio DOUBLE PRECISION,
    max_pain_strike DOUBLE PRECISION,
    total_open_interest BIGINT,
    total_volume BIGINT,
    total_gex DOUBLE PRECISION,
    iv_atm DOUBLE PRECISION,
    source_payload JSONB NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_options_snapshot_history_asof
    ON market.options_snapshot_history (symbol, observed_at DESC, snapshot_id DESC);

CREATE TABLE IF NOT EXISTS market.options_contract_history (
    snapshot_id BIGINT NOT NULL REFERENCES market.options_snapshot_history(snapshot_id) ON DELETE CASCADE,
    contract_symbol TEXT NOT NULL,
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
    PRIMARY KEY (snapshot_id, contract_symbol)
);

CREATE INDEX IF NOT EXISTS idx_options_contract_history_symbol
    ON market.options_contract_history (symbol, expiration_date, strike);

CREATE TABLE IF NOT EXISTS macro.macro_observation_history (
    history_id BIGSERIAL PRIMARY KEY,
    event_id TEXT NOT NULL,
    source TEXT NOT NULL,
    observation_type TEXT NOT NULL CHECK (observation_type IN ('rate', 'spread', 'series')),
    series_key TEXT NOT NULL,
    observation_date DATE NOT NULL,
    value DOUBLE PRECISION NOT NULL,
    raw_value TEXT,
    available_at TIMESTAMPTZ NOT NULL,
    source_payload JSONB NOT NULL,
    UNIQUE (event_id, observation_type)
);

CREATE INDEX IF NOT EXISTS idx_macro_observation_history_asof
    ON macro.macro_observation_history (observation_type, series_key, observation_date, available_at DESC);

GRANT SELECT, INSERT ON market.options_snapshot_history, market.options_contract_history TO market_user;
GRANT USAGE, SELECT ON SEQUENCE market.options_snapshot_history_snapshot_id_seq TO market_user;
GRANT SELECT ON market.options_snapshot_history, market.options_contract_history TO intelligence_user, readonly_slave_user;

GRANT SELECT, INSERT ON macro.macro_observation_history TO macro_sink_user;
GRANT USAGE, SELECT ON SEQUENCE macro.macro_observation_history_history_id_seq TO macro_sink_user;
GRANT SELECT ON macro.macro_observation_history TO intelligence_user, readonly_slave_user;
