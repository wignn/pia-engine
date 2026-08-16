CREATE TABLE IF NOT EXISTS market.ohlcv_1m
(
    symbol LowCardinality(String),
    minute DateTime64(3, 'UTC'),
    open_state AggregateFunction(argMin, Float64, DateTime64(3, 'UTC')),
    high_state AggregateFunction(max, Float64),
    low_state AggregateFunction(min, Float64),
    close_state AggregateFunction(argMax, Float64, DateTime64(3, 'UTC')),
    volume_state AggregateFunction(sum, Float64)
)
ENGINE = AggregatingMergeTree
PARTITION BY toYYYYMM(minute)
ORDER BY (symbol, minute)
TTL toDateTime(minute) + INTERVAL 365 DAY
SETTINGS index_granularity = 8192;

CREATE MATERIALIZED VIEW IF NOT EXISTS market.ohlcv_1m_mv
TO market.ohlcv_1m
AS
SELECT
    symbol,
    toDateTime64(toStartOfMinute(time), 3, 'UTC') AS minute,
    argMinState(price, time) AS open_state,
    maxState(price) AS high_state,
    minState(price) AS low_state,
    argMaxState(price, time) AS close_state,
    sumState(volume) AS volume_state
FROM market.price_ticks
GROUP BY symbol, minute;

-- Higher resolutions also straight from ticks into the existing (empty)
-- aggregate tables. The old 5m/15m/1h MVs sourced from ohlcv_candles stay
-- dormant because nothing inserts into that table anymore.
CREATE MATERIALIZED VIEW IF NOT EXISTS market.ohlcv_5m_from_ticks_mv
TO market.ohlcv_candles_5m
AS
SELECT
    symbol,
    '5m' AS resolution,
    toDateTime64(toStartOfInterval(time, INTERVAL 5 MINUTE), 3, 'UTC') AS time,
    argMinState(price, time) AS open_state,
    maxState(price) AS high_state,
    minState(price) AS low_state,
    argMaxState(price, time) AS close_state,
    sumState(volume) AS volume_state
FROM market.price_ticks
GROUP BY symbol, time;

CREATE MATERIALIZED VIEW IF NOT EXISTS market.ohlcv_15m_from_ticks_mv
TO market.ohlcv_candles_15m
AS
SELECT
    symbol,
    '15m' AS resolution,
    toDateTime64(toStartOfInterval(time, INTERVAL 15 MINUTE), 3, 'UTC') AS time,
    argMinState(price, time) AS open_state,
    maxState(price) AS high_state,
    minState(price) AS low_state,
    argMaxState(price, time) AS close_state,
    sumState(volume) AS volume_state
FROM market.price_ticks
GROUP BY symbol, time;

CREATE MATERIALIZED VIEW IF NOT EXISTS market.ohlcv_1h_from_ticks_mv
TO market.ohlcv_candles_1h
AS
SELECT
    symbol,
    '1h' AS resolution,
    toDateTime64(toStartOfInterval(time, INTERVAL 1 HOUR), 3, 'UTC') AS time,
    argMinState(price, time) AS open_state,
    maxState(price) AS high_state,
    minState(price) AS low_state,
    argMaxState(price, time) AS close_state,
    sumState(volume) AS volume_state
FROM market.price_ticks
GROUP BY symbol, time;
