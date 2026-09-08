# Institutional Market Data Pre-Aggregation & Tiering Implementation Plan

> **For Hermes:** Use `subagent-driven-development` or step-by-step TDD execution to implement this plan task-by-task.

**Goal:** Transform the ATLSD market data read path into an institutional-grade multi-tier architecture using ClickHouse Materialized Views (`AggregatingMergeTree`), application-level Request Coalescing (SingleFlight), and bounded in-memory micro-caching so that the system sustains 200+ HTTP req/s and 200+ concurrent WebSockets with sub-15ms p95 latency and zero database saturation.

**Architecture:**
1. **Analytical Database Layer (ClickHouse CQRS)**: Pre-aggregate raw tick streams (`market.price_ticks`) into 1-minute OHLCV rollups (`market.candles_1m_v2`) using `AggregatingMergeTree` and ClickHouse `MATERIALIZED VIEW`. Higher timeframes (`5m`, `15m`, `1h`, `1d`) query the 1-minute rollup table instead of scanning millions of raw ticks.
2. **Coalescing & Micro-Cache Layer (`market-data` service)**: Deploy in-memory bounded LRU caching (`moka`) with dual TTL (2.5s for live active candles, 60s for historical immutable pages) combined with Tokio-based Request Coalescing (SingleFlight) to prevent Thundering Herd on ClickHouse during volatility spikes.
3. **Dual-Tier Candle Synthesis**: Merge closed historical candles from ClickHouse/cache with in-memory active minute candles from the live tick buffer (`CachedPrice`) for real-time responsiveness.

**Tech Stack:**
- ClickHouse (`AggregatingMergeTree`, `SimpleAggregateFunction`, `AggregateFunction(argMin/argMax)`)
- Rust / Tokio / Axum / `moka`
- Grafana k6 (Load testing verification)

---

### Task 1: ClickHouse Pre-Aggregated OHLCV Schema & Materialized View Migration

**Objective:** Create the institutional 1-minute pre-aggregated rollup table (`market.candles_1m_v2`) and the automatic Materialized View connected to `market.price_ticks`, plus backfill historical candles from existing ticks.

**Files:**
- Create: `db/migrations/clickhouse/002_preaggregated_candles.sql`

**Step 1: Author the migration SQL script**

```sql
-- db/migrations/clickhouse/002_preaggregated_candles.sql
CREATE TABLE IF NOT EXISTS market.candles_1m_v2
(
    symbol LowCardinality(String),
    bucket_time DateTime,
    open_state AggregateFunction(argMin, Float64, DateTime64(3, 'UTC')),
    high_state SimpleAggregateFunction(max, Float64),
    low_state SimpleAggregateFunction(min, Float64),
    close_state AggregateFunction(argMax, Float64, DateTime64(3, 'UTC')),
    volume_state SimpleAggregateFunction(sum, Float64),
    tick_count_state SimpleAggregateFunction(count, UInt64)
)
ENGINE = AggregatingMergeTree()
PARTITION BY toYYYYMM(bucket_time)
ORDER BY (symbol, bucket_time)
TTL toDateTime(bucket_time) + INTERVAL 90 DAY
SETTINGS index_granularity = 8192;

-- Materialized view to automatically roll up new ticks on insert
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
    count() AS tick_count_state
FROM market.price_ticks
WHERE price > 0
GROUP BY symbol, bucket_time;
```

**Step 2: Apply migration to live ClickHouse container**

Run:
```bash
docker exec -i clickhouse-prod clickhouse-client < db/migrations/clickhouse/002_preaggregated_candles.sql
```
Expected: Exit code 0, tables `market.candles_1m_v2` and view `market.mv_price_ticks_to_candles_1m` created.

**Step 3: Backfill historical candles from existing `market.price_ticks`**

Run:
```bash
docker exec -i clickhouse-prod clickhouse-client --query "
INSERT INTO market.candles_1m_v2
SELECT
    symbol,
    toDateTime(toStartOfInterval(time, INTERVAL 1 MINUTE)) AS bucket_time,
    argMinState(price, time) AS open_state,
    max(price) AS high_state,
    min(price) AS low_state,
    argMaxState(price, time) AS close_state,
    sum(volume) AS volume_state,
    count() AS tick_count_state
FROM market.price_ticks
WHERE price > 0
GROUP BY symbol, bucket_time;
"
```
Expected: Exit code 0. Verify with:
`docker exec clickhouse-prod clickhouse-client --query "SELECT count() FROM market.candles_1m_v2"` (returns tens of thousands of rows instead of 0).

**Step 4: Commit migration**

```bash
git add db/migrations/clickhouse/002_preaggregated_candles.sql
git commit -m "feat(db): add clickhouse pre-aggregated 1m candle rollup and materialized view"
```

---

### Task 2: ClickHouse Client Query Engine Refactoring

**Objective:** Update `services/market-data/src/clickhouse.rs` to read from `market.candles_1m_v2` with `argMinMerge / argMaxMerge` rather than running heavy subqueries over `market.price_ticks`.

**Files:**
- Modify: `services/market-data/src/clickhouse.rs`
- Test: `services/market-data/tests/clickhouse_queries.rs`

**Step 1: Write query unit test for SQL generation**

Test that `latest_history_sql` constructs valid, index-friendly AggregatingMergeTree queries without nested `(SELECT max(time)...)` subqueries:
```rust
#[test]
fn test_history_sql_uses_preaggregated_table() {
    let sql = latest_history_sql("market", "BTCUSDT", "1m", 120, None);
    assert!(sql.contains("market.candles_1m_v2"));
    assert!(sql.contains("argMinMerge(open_state) AS open"));
    assert!(!sql.contains("(SELECT max(time) FROM"));
}
```

**Step 2: Run test to verify failure**

Run: `cargo test -p market-data test_history_sql_uses_preaggregated_table`
Expected: FAIL — currently generates raw `price_ticks` queries.

**Step 3: Implement optimized SQL generator in `clickhouse.rs`**

```rust
fn latest_history_sql(
    database: &str,
    symbol: &str,
    resolution: &str,
    limit: usize,
    before: Option<i64>,
) -> String {
    let bucket_minutes = history_bucket_minutes(resolution);
    let bucket_interval = clickhouse_bucket_interval(bucket_minutes);
    let bounded_limit = limit.clamp(1, 1000);

    let where_clause = match before {
        Some(ts) => format!("symbol = {} AND bucket_time < toDateTime({ts})", string_literal(symbol)),
        None => format!("symbol = {}", string_literal(symbol)),
    };

    if bucket_minutes == 1 {
        format!(
            "SELECT \
                toUnixTimestamp(bucket_time) AS time, \
                argMaxMerge(close_state) AS value, \
                argMinMerge(open_state) AS open, \
                max(high_state) AS high, \
                min(low_state) AS low, \
                argMaxMerge(close_state) AS close, \
                sum(volume_state) AS volume, \
                sum(tick_count_state) AS tick_count, \
                toString(max(bucket_time)) AS latest_at, \
                'clickhouse_candles_1m' AS source \
             FROM {}.candles_1m_v2 \
             WHERE {} \
             GROUP BY bucket_time \
             ORDER BY bucket_time DESC \
             LIMIT {} FORMAT JSONEachRow",
            ident(database),
            where_clause,
            bounded_limit
        )
    } else {
        // Roll up higher timeframes (5m, 15m, 1h, 1d) from 1m candles
        format!(
            "SELECT \
                toUnixTimestamp(toStartOfInterval(bucket_time, {})) AS time, \
                argMaxMerge(close_state) AS value, \
                argMinMerge(open_state) AS open, \
                max(high_state) AS high, \
                min(low_state) AS low, \
                argMaxMerge(close_state) AS close, \
                sum(volume_state) AS volume, \
                sum(tick_count_state) AS tick_count, \
                toString(max(bucket_time)) AS latest_at, \
                'clickhouse_candles_rollup' AS source \
             FROM {}.candles_1m_v2 \
             WHERE {} \
             GROUP BY time \
             ORDER BY time DESC \
             LIMIT {} FORMAT JSONEachRow",
            bucket_interval,
            ident(database),
            where_clause,
            bounded_limit
        )
    }
}
```

**Step 4: Run unit test to verify pass**

Run: `cargo test -p market-data test_history_sql_uses_preaggregated_table`
Expected: PASS.

**Step 5: Commit changes**

```bash
git add services/market-data/src/clickhouse.rs
git commit -m "perf(market-data): query pre-aggregated clickhouse candles with argMinMerge"
```

---

### Task 3: In-Memory Bounded Micro-Cache & Request Coalescing (SingleFlight)

**Objective:** Implement an in-memory cache using `moka` with dual TTL and a Tokio `broadcast`-based SingleFlight coalescer in `services/market-data` to eliminate redundant database queries under high concurrency.

**Files:**
- Modify: `services/market-data/Cargo.toml`
- Create: `services/market-data/src/cache.rs`
- Modify: `services/market-data/src/main.rs`
- Modify: `services/market-data/src/history.rs`

**Step 1: Add `moka` dependency to `services/market-data/Cargo.toml`**

```toml
moka = { version = "0.12", features = ["future"] }
```

**Step 2: Create `services/market-data/src/cache.rs` with SingleFlight & LRU**

```rust
use std::sync::Arc;
use std::time::Duration;
use moka::future::Cache;
use tokio::sync::{broadcast, Mutex};
use std::collections::HashMap;
use crate::clickhouse::CandleRow;

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct HistoryCacheKey {
    pub symbol: String,
    pub resolution: String,
    pub limit: usize,
    pub before: Option<i64>,
}

#[derive(Clone)]
pub struct MarketHistoryCache {
    live_cache: Cache<HistoryCacheKey, Arc<Vec<CandleRow>>>,
    historical_cache: Cache<HistoryCacheKey, Arc<Vec<CandleRow>>>,
    in_flight: Arc<Mutex<HashMap<HistoryCacheKey, broadcast::Sender<Arc<Vec<CandleRow>>>>>>,
}

impl MarketHistoryCache {
    pub fn new() -> Self {
        Self {
            // Live active history: TTL 2.5 seconds, max 2,000 entries
            live_cache: Cache::builder()
                .time_to_live(Duration::from_millis(2500))
                .max_capacity(2000)
                .build(),
            // Historical cursor pages: TTL 120 seconds, max 5,000 entries
            historical_cache: Cache::builder()
                .time_to_live(Duration::from_secs(120))
                .max_capacity(5000)
                .build(),
            in_flight: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_or_fetch<F, Fut>(&self, key: HistoryCacheKey, fetch: F) -> Result<Arc<Vec<CandleRow>>, String>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<Vec<CandleRow>, String>>,
    {
        let is_live = key.before.is_none();
        let target_cache = if is_live { &self.live_cache } else { &self.historical_cache };

        if let Some(cached) = target_cache.get(&key).await {
            return Ok(cached);
        }

        // SingleFlight coalescing logic
        let mut rx = {
            let mut in_flight = self.in_flight.lock().await;
            if let Some(sender) = in_flight.get(&key) {
                sender.subscribe()
            } else {
                let (tx, rx) = broadcast::channel(1);
                in_flight.insert(key.clone(), tx);
                drop(in_flight);

                // We are the leader for this key
                let result = fetch().await;
                let mut in_flight = self.in_flight.lock().await;
                let sender = in_flight.remove(&key);

                match result {
                    Ok(data) => {
                        let arc_data = Arc::new(data);
                        target_cache.insert(key, arc_data.clone()).await;
                        if let Some(tx) = sender {
                            let _ = tx.send(arc_data.clone());
                        }
                        return Ok(arc_data);
                    }
                    Err(err) => {
                        return Err(err);
                    }
                }
            }
        };

        // Follower request: wait for the leader's broadcast
        match rx.recv().await {
            Ok(data) => Ok(data),
            Err(_) => Err("coalesced request closed".to_string()),
        }
    }
}
```

**Step 3: Integrate `MarketHistoryCache` into `AppState` in `src/main.rs` and `src/history.rs`**

Replace direct unthrottled ClickHouse query in `src/history.rs` with `state.history_cache.get_or_fetch(key, ...)`.

**Step 4: Compile and test workspace**

Run: `cargo test -p market-data`
Expected: PASS.

**Step 5: Commit changes**

```bash
git add services/market-data/Cargo.toml services/market-data/src/cache.rs services/market-data/src/main.rs services/market-data/src/history.rs
git commit -m "feat(market-data): add singleflight request coalescing and bounded moka micro-cache"
```

---

### Task 4: Dual-Tier In-Flight Active Candle Stitching

**Objective:** Stitch the active (incomplete) minute candle from the in-memory tick buffer (`prices.rs`) onto the historical closed candles returned by ClickHouse so clients immediately see tick updates without waiting for ClickHouse flushes.

**Files:**
- Modify: `services/market-data/src/history.rs`

**Step 1: Write unit test for active candle stitching**

Verify that when `before == None` and a live tick exists newer than the latest candle from ClickHouse, a synthesized candle with the latest price and volume is appended to the response items.

**Step 2: Implement stitching in `history.rs`**

```rust
if query.before.is_none() && resolution == "1m" {
    if let Some(cached_price) = state.prices.get(&symbol) {
        if let Some(latest_candle) = items.first_mut() {
            let current_minute_bucket = (cached_price.time.timestamp() / 60) * 60;
            if latest_candle.time == current_minute_bucket {
                latest_candle.close = cached_price.price;
                latest_candle.high = latest_candle.high.max(cached_price.price);
                latest_candle.low = latest_candle.low.min(cached_price.price);
                latest_candle.latest_at = cached_price.time.to_rfc3339();
            }
        }
    }
}
```

**Step 3: Verify test pass**

Run: `cargo test -p market-data`
Expected: PASS.

**Step 4: Commit changes**

```bash
git add services/market-data/src/history.rs
git commit -m "feat(market-data): stitch in-memory active candle into latest history responses"
```

---

### Task 5: End-to-End Build, Deploy, and 200 WS / 200 RPS Load Test Verification

**Objective:** Rebuild `services/market-data`, deploy cleanly to the blue-green stack, and run k6 load test to verify 0 dropped connections and < 15ms p95 latency under 200 req/s.

**Files:**
- Test: `loadtest_200ws_200rps.js`

**Step 1: Execute mandatory pre-push linters**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```
Expected: PASS with 0 warnings.

**Step 2: Push and trigger blue-green deploy**

```bash
git push origin main
```
Observe CI deploy or trigger blue-green rotation on host.

**Step 3: Execute full load test with k6**

Run:
```bash
k6 run /home/wign/atlsd/loadtest_200ws_200rps.js
```
Expected output:
- `http_req_failed` < 0.1% (Zero 503 errors).
- `http_req_duration p(95)` < 15ms (down from 10+ seconds timeout).
- `ws_session_duration` sustained for 200 connections over 5m.
- ClickHouse active processes (`system.processes`) stay < 5 at all times.

---

## Execution Handoff

Plan complete and saved to `.hermes/plans/2026-09-08_114355-institutional-market-data-tiering.md`.
Ready to execute task-by-task with strict pre-push verification (`cargo fmt`, `clippy`, and real load test run). Shall I proceed?
