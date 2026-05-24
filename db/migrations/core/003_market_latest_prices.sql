CREATE TABLE IF NOT EXISTS market_latest_prices (
    symbol TEXT PRIMARY KEY,
    price DOUBLE PRECISION NOT NULL CHECK (price > 0),
    bid DOUBLE PRECISION,
    ask DOUBLE PRECISION,
    volume DOUBLE PRECISION,
    source TEXT NOT NULL DEFAULT 'market_data',
    asset_type TEXT NOT NULL DEFAULT 'unknown',
    received_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_market_latest_prices_asset_type
    ON market_latest_prices(asset_type);

CREATE INDEX IF NOT EXISTS idx_market_latest_prices_updated_at
    ON market_latest_prices(updated_at DESC);

INSERT INTO market_latest_prices (symbol, price, volume, source, asset_type, received_at, updated_at)
SELECT DISTINCT ON (symbol)
    symbol,
    close AS price,
    volume,
    'ohlcv_candles' AS source,
    'unknown' AS asset_type,
    time AS received_at,
    time AS updated_at
FROM ohlcv_candles
WHERE resolution = '1m'
  AND close > 0
ORDER BY symbol, time DESC
ON CONFLICT (symbol) DO NOTHING;
