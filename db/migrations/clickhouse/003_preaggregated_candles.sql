-- ClickHouse Migration: Pre-aggregated 1-Minute OHLCV Rollup & Materialized View
CREATE TABLE IF NOT EXISTS market.candles_1m_v2
(
    symbol LowCardinality(String),
    bucket_time DateTime,
    open_state AggregateFunction(argMin, Float64, DateTime64(3, 'UTC')),
    high_state SimpleAggregateFunction(max, Float64),
    low_state SimpleAggregateFunction(min, Float64),
    close_state AggregateFunction(argMax, Float64, DateTime64(3, 'UTC')),
    volume_state SimpleAggregateFunction(sum, Float64),
    tick_count SimpleAggregateFunction(sum, UInt64)
)
ENGINE = AggregatingMergeTree()
PARTITION BY toYYYYMM(bucket_time)
ORDER BY (symbol, bucket_time)
TTL toDateTime(bucket_time) + INTERVAL 90 DAY
SETTINGS index_granularity = 8192;

-- Materialized View: automatically roll up new ticks on insert
CREATE MATERIALIZED VIEW IF NOT EXISTS market.mv_price_ticks_to_candles_1m
TO market.candles_1m_v2
AS
SELECT
    symbol,
    toDateTime(toStartOfInterval(time, INTERVAL 1 MINUTE)) AS bucket_time,
    argMinState(price, time) AS open_state,
    max(price) AS high_state,
    min(price) AS low_state,
    argMaxState(price, time) AS close_state,
    sum(volume) AS volume_state,
    toUInt64(count()) AS tick_count
FROM market.price_ticks
WHERE price > 0
GROUP BY symbol, bucket_time;
