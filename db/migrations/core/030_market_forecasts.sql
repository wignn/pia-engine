DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'model_worker_user') THEN
        CREATE ROLE model_worker_user WITH LOGIN;
    END IF;
END
$$;

ALTER ROLE model_worker_user SET search_path TO market, news, macro, public;
GRANT USAGE ON SCHEMA market, news, macro TO model_worker_user;

CREATE TABLE IF NOT EXISTS market.market_forecasts (
    forecast_id BIGSERIAL PRIMARY KEY,
    symbol TEXT NOT NULL CHECK (symbol = 'XAUUSD'),
    timeframe TEXT NOT NULL CHECK (timeframe = '15m'),
    decision_at TIMESTAMPTZ NOT NULL,
    horizon_end TIMESTAMPTZ NOT NULL,
    reference_price DOUBLE PRECISION NOT NULL CHECK (reference_price > 0),
    down_probability DOUBLE PRECISION NOT NULL CHECK (down_probability BETWEEN 0 AND 1),
    flat_probability DOUBLE PRECISION NOT NULL CHECK (flat_probability BETWEEN 0 AND 1),
    up_probability DOUBLE PRECISION NOT NULL CHECK (up_probability BETWEEN 0 AND 1),
    expected_return DOUBLE PRECISION NOT NULL,
    q10_return DOUBLE PRECISION NOT NULL,
    q50_return DOUBLE PRECISION NOT NULL,
    q90_return DOUBLE PRECISION NOT NULL,
    uncertainty DOUBLE PRECISION NOT NULL CHECK (uncertainty >= 0),
    model_version TEXT NOT NULL,
    data_version TEXT NOT NULL,
    feature_version TEXT NOT NULL,
    source_masks JSONB NOT NULL DEFAULT '{}'::jsonb,
    source_age_seconds JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT market_forecasts_probability_sum CHECK (
        abs(down_probability + flat_probability + up_probability - 1.0) < 0.0001
    ),
    CONSTRAINT market_forecasts_horizon CHECK (horizon_end > decision_at),
    CONSTRAINT market_forecasts_model_decision_unique UNIQUE (model_version, symbol, decision_at)
);

CREATE INDEX IF NOT EXISTS idx_market_forecasts_latest
    ON market.market_forecasts (symbol, timeframe, decision_at DESC);

GRANT SELECT, INSERT ON market.market_forecasts TO model_worker_user;
GRANT USAGE, SELECT ON SEQUENCE market.market_forecasts_forecast_id_seq TO model_worker_user;
GRANT SELECT ON market.market_forecasts TO market_user, intelligence_user, readonly_slave_user;

GRANT SELECT ON news.forex_news_articles, news.forex_news_analyses, news.social_posts,
                news.geosignals, news.central_bank_documents TO model_worker_user;
GRANT SELECT ON market.options_snapshot_history, market.options_contract_history TO model_worker_user;
GRANT SELECT ON macro.macro_observation_history, macro.economic_calendar_events,
                macro.cot_reports, macro.fear_greed_index TO model_worker_user;
