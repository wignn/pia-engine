CREATE DATABASE IF NOT EXISTS market;

CREATE TABLE IF NOT EXISTS market.price_ticks
(
    symbol LowCardinality(String),
    time DateTime64(3, 'UTC'),
    price Float64,
    bid Nullable(Float64),
    ask Nullable(Float64),
    volume Float64 DEFAULT 0,
    source LowCardinality(String),
    asset_type LowCardinality(String)
)
ENGINE = MergeTree
PARTITION BY toYYYYMM(time)
ORDER BY (symbol, time)
TTL toDateTime(time) + INTERVAL 30 DAY
SETTINGS index_granularity = 8192;

CREATE TABLE IF NOT EXISTS market.ohlcv_candles
(
    symbol LowCardinality(String),
    resolution LowCardinality(String),
    time DateTime64(3, 'UTC'),
    open Float64,
    high Float64,
    low Float64,
    close Float64,
    volume Float64 DEFAULT 0,
    updated_at DateTime64(3, 'UTC') DEFAULT now64(3)
)
ENGINE = ReplacingMergeTree(updated_at)
PARTITION BY toYYYYMM(time)
ORDER BY (symbol, resolution, time)
SETTINGS index_granularity = 8192;

CREATE TABLE IF NOT EXISTS market.ohlcv_candles_5m
(
    symbol LowCardinality(String),
    resolution LowCardinality(String),
    time DateTime64(3, 'UTC'),
    open_state AggregateFunction(argMin, Float64, DateTime64(3, 'UTC')),
    high_state AggregateFunction(max, Float64),
    low_state AggregateFunction(min, Float64),
    close_state AggregateFunction(argMax, Float64, DateTime64(3, 'UTC')),
    volume_state AggregateFunction(sum, Float64)
)
ENGINE = AggregatingMergeTree
PARTITION BY toYYYYMM(time)
ORDER BY (symbol, resolution, time)
SETTINGS index_granularity = 8192;

CREATE TABLE IF NOT EXISTS market.ohlcv_candles_15m
(
    symbol LowCardinality(String),
    resolution LowCardinality(String),
    time DateTime64(3, 'UTC'),
    open_state AggregateFunction(argMin, Float64, DateTime64(3, 'UTC')),
    high_state AggregateFunction(max, Float64),
    low_state AggregateFunction(min, Float64),
    close_state AggregateFunction(argMax, Float64, DateTime64(3, 'UTC')),
    volume_state AggregateFunction(sum, Float64)
)
ENGINE = AggregatingMergeTree
PARTITION BY toYYYYMM(time)
ORDER BY (symbol, resolution, time)
SETTINGS index_granularity = 8192;

CREATE TABLE IF NOT EXISTS market.ohlcv_candles_1h
(
    symbol LowCardinality(String),
    resolution LowCardinality(String),
    time DateTime64(3, 'UTC'),
    open_state AggregateFunction(argMin, Float64, DateTime64(3, 'UTC')),
    high_state AggregateFunction(max, Float64),
    low_state AggregateFunction(min, Float64),
    close_state AggregateFunction(argMax, Float64, DateTime64(3, 'UTC')),
    volume_state AggregateFunction(sum, Float64)
)
ENGINE = AggregatingMergeTree
PARTITION BY toYYYYMM(time)
ORDER BY (symbol, resolution, time)
SETTINGS index_granularity = 8192;

CREATE MATERIALIZED VIEW IF NOT EXISTS market.ohlcv_candles_5m_mv
TO market.ohlcv_candles_5m
AS
SELECT
    symbol,
    '5m' AS resolution,
    bucket_time AS time,
    argMinState(open, source_time) AS open_state,
    maxState(high) AS high_state,
    minState(low) AS low_state,
    argMaxState(close, source_time) AS close_state,
    sumState(volume) AS volume_state
FROM
(
    SELECT
        symbol,
        time AS source_time,
        toDateTime64(toStartOfInterval(time, INTERVAL 5 MINUTE), 3, 'UTC') AS bucket_time,
        open,
        high,
        low,
        close,
        volume
    FROM market.ohlcv_candles
    WHERE resolution = '1m'
)
GROUP BY symbol, bucket_time;

CREATE MATERIALIZED VIEW IF NOT EXISTS market.ohlcv_candles_15m_mv
TO market.ohlcv_candles_15m
AS
SELECT
    symbol,
    '15m' AS resolution,
    bucket_time AS time,
    argMinState(open, source_time) AS open_state,
    maxState(high) AS high_state,
    minState(low) AS low_state,
    argMaxState(close, source_time) AS close_state,
    sumState(volume) AS volume_state
FROM
(
    SELECT
        symbol,
        time AS source_time,
        toDateTime64(toStartOfInterval(time, INTERVAL 15 MINUTE), 3, 'UTC') AS bucket_time,
        open,
        high,
        low,
        close,
        volume
    FROM market.ohlcv_candles
    WHERE resolution = '1m'
)
GROUP BY symbol, bucket_time;

CREATE MATERIALIZED VIEW IF NOT EXISTS market.ohlcv_candles_1h_mv
TO market.ohlcv_candles_1h
AS
SELECT
    symbol,
    '1h' AS resolution,
    bucket_time AS time,
    argMinState(open, source_time) AS open_state,
    maxState(high) AS high_state,
    minState(low) AS low_state,
    argMaxState(close, source_time) AS close_state,
    sumState(volume) AS volume_state
FROM
(
    SELECT
        symbol,
        time AS source_time,
        toDateTime64(toStartOfInterval(time, INTERVAL 1 HOUR), 3, 'UTC') AS bucket_time,
        open,
        high,
        low,
        close,
        volume
    FROM market.ohlcv_candles
    WHERE resolution = '1m'
)
GROUP BY symbol, bucket_time;
