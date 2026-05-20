-- OHLCV Candles table for historical time-series chart data
CREATE TABLE IF NOT EXISTS ohlcv_candles (
    symbol      TEXT NOT NULL,
    resolution  TEXT NOT NULL, -- e.g. '1m', '5m', '1h', '1d'
    time        TIMESTAMPTZ NOT NULL,
    open        DOUBLE PRECISION NOT NULL,
    high        DOUBLE PRECISION NOT NULL,
    low         DOUBLE PRECISION NOT NULL,
    close       DOUBLE PRECISION NOT NULL,
    volume      DOUBLE PRECISION NOT NULL DEFAULT 0,
    PRIMARY KEY (symbol, resolution, time)
);

CREATE INDEX IF NOT EXISTS idx_ohlcv_candles_time ON ohlcv_candles(time DESC);

-- Optional TimescaleDB optimization if available:
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'timescaledb') THEN
        PERFORM create_hypertable('ohlcv_candles', 'time', if_not_exists => TRUE);
    END IF;
EXCEPTION
    WHEN OTHERS THEN
        NULL;
END $$;
