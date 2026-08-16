INSERT INTO market.ohlcv_1m
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

INSERT INTO market.ohlcv_candles_5m
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

INSERT INTO market.ohlcv_candles_15m
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

INSERT INTO market.ohlcv_candles_1h
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
