ALTER TABLE public.market_latest_prices
    ADD COLUMN IF NOT EXISTS provider_ts_ms BIGINT;
