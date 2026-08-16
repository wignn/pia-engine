use chrono::{DateTime, TimeZone, Utc};
use std::collections::HashMap;
use std::sync::Arc;

use atlsd_contracts::market::{AssetClass, CandleResolution, OhlcvCandle};
use atlsd_eventbus::EventPublisher;

pub struct CandleEngine {
    grace: chrono::Duration,
    candles: HashMap<(String, DateTime<Utc>), CandleEntry>,
    sequences: HashMap<(String, DateTime<Utc>), u64>,
    published_boundaries: HashMap<String, DateTime<Utc>>,
}

#[derive(Clone)]
struct CandleEntry {
    open: (i64, f64),
    high: f64,
    low: f64,
    close: (i64, f64),
    volume: f64,
    tick_count: u64,
    published: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickDisposition {
    Applied,
    NoTimestamp,
    LateApplied,
}

impl CandleEngine {
    pub fn new(grace_secs: u64) -> Self {
        Self {
            grace: chrono::Duration::seconds(grace_secs as i64),
            candles: HashMap::new(),
            sequences: HashMap::new(),
            published_boundaries: HashMap::new(),
        }
    }

    pub fn accept_tick(
        &mut self,
        symbol: &str,
        provider_ts_ms: Option<i64>,
        price: f64,
        volume: f64,
        asset_class: AssetClass,
        now: DateTime<Utc>,
    ) -> (TickDisposition, Option<OhlcvCandle>) {
        let Some(ts_ms) = provider_ts_ms else {
            return (TickDisposition::NoTimestamp, None);
        };
        if price <= 0.0 {
            return (TickDisposition::NoTimestamp, None);
        }

        let minute = minute_bucket_from_ms(ts_ms);
        let entry = self
            .candles
            .entry((symbol.to_string(), minute))
            .or_insert_with(|| CandleEntry {
                open: (ts_ms, price),
                high: price,
                low: price,
                close: (ts_ms, price),
                volume: 0.0,
                tick_count: 0,
                published: false,
            });

        if ts_ms < entry.open.0 {
            entry.open = (ts_ms, price);
        }
        if ts_ms > entry.close.0 {
            entry.close = (ts_ms, price);
        }
        entry.high = entry.high.max(price);
        entry.low = entry.low.min(price);
        entry.volume += volume;
        entry.tick_count += 1;

        if entry.published {
            let snapshot = entry.clone();
            let candle = self.freeze(symbol, minute, asset_class, &snapshot, now, true);
            (TickDisposition::LateApplied, Some(candle))
        } else {
            (TickDisposition::Applied, None)
        }
    }

    pub fn collect_due(&mut self, now: DateTime<Utc>) -> Vec<OhlcvCandle> {
        let mut due: Vec<(String, DateTime<Utc>)> = self
            .candles
            .iter()
            .filter(|(_, entry)| !entry.published)
            .filter(|((_, minute), _)| *minute + self.grace + chrono::Duration::seconds(60) <= now)
            .map(|((symbol, minute), _)| (symbol.clone(), *minute))
            .collect();
        due.sort_by_key(|(_, minute)| *minute);

        let mut events = Vec::with_capacity(due.len());
        for (symbol, minute) in due {
            let Some(entry) = self.candles.get_mut(&(symbol.clone(), minute)) else {
                continue;
            };
            if entry.published {
                continue;
            }
            entry.published = true;
            let snapshot = entry.clone();
            let asset_class = AssetClass::Unknown;
            let candle = self.freeze(&symbol, minute, asset_class, &snapshot, now, false);
            self.published_boundaries.insert(symbol.clone(), minute);
            events.push(candle);
        }
        events
    }

    fn freeze(
        &mut self,
        symbol: &str,
        minute: DateTime<Utc>,
        asset_class: AssetClass,
        entry: &CandleEntry,
        now: DateTime<Utc>,
        corrected: bool,
    ) -> OhlcvCandle {
        let key = (symbol.to_string(), minute);
        let sequence = {
            let counter = self.sequences.entry(key).or_insert(0);
            *counter += 1;
            *counter
        };

        OhlcvCandle {
            symbol: symbol.to_string(),
            asset_class,
            resolution: CandleResolution::OneMinute,
            bucket_start: minute,
            open: entry.open.1,
            high: entry.high,
            low: entry.low,
            close: entry.close.1,
            volume: entry.volume,
            tick_count: entry.tick_count,
            corrected,
            sequence,
            closed_at: now,
        }
    }

    pub fn evict_stale(&mut self, keep_minutes: i64) {
        let boundaries: Vec<(String, DateTime<Utc>)> = self
            .published_boundaries
            .iter()
            .map(|(symbol, minute)| (symbol.clone(), *minute))
            .collect();
        for (symbol, boundary) in boundaries {
            let cutoff = boundary - chrono::Duration::minutes(keep_minutes);
            self.candles
                .retain(|(s, minute), _| !(s == &symbol && *minute < cutoff));
            self.sequences
                .retain(|(s, minute), _| !(s == &symbol && *minute < cutoff));
        }
    }
}

pub fn minute_bucket_from_ms(ts_ms: i64) -> DateTime<Utc> {
    let bucketed = (ts_ms / 60_000) * 60_000;
    Utc.timestamp_millis_opt(bucketed)
        .single()
        .unwrap_or_else(Utc::now)
}

// ---------------------------------------------------------------------------
// Runtime wiring: shared handle + collector timer + JetStream publisher.
// ---------------------------------------------------------------------------

pub struct CandleEngineHandle {
    engine: parking_lot::Mutex<CandleEngine>,
    events: tokio::sync::mpsc::Sender<OhlcvCandle>,
    metrics: Option<Arc<atlsd_observability::MetricsRegistry>>,
}

impl CandleEngineHandle {
    pub fn new(
        grace_secs: u64,
        metrics: Option<Arc<atlsd_observability::MetricsRegistry>>,
    ) -> (Arc<Self>, tokio::sync::mpsc::Receiver<OhlcvCandle>) {
        let (events, rx) = tokio::sync::mpsc::channel(1000);
        (
            Arc::new(Self {
                engine: parking_lot::Mutex::new(CandleEngine::new(grace_secs)),
                events,
                metrics,
            }),
            rx,
        )
    }

    fn record(&self, name: &str, help: &str) {
        if let Some(metrics) = &self.metrics {
            metrics.inc(name, help);
        }
    }

    pub fn accept_tick(
        &self,
        symbol: &str,
        provider_ts_ms: Option<i64>,
        price: f64,
        volume: f64,
        asset_type: &str,
        now: DateTime<Utc>,
    ) {
        let (disposition, correction) = self.engine.lock().accept_tick(
            symbol,
            provider_ts_ms,
            price,
            volume,
            asset_class(asset_type),
            now,
        );
        match disposition {
            TickDisposition::NoTimestamp => {
                self.record(
                    "atlsd_market_ticks_no_ts_total",
                    "Ticks excluded from candles: no provider timestamp.",
                );
                tracing::debug!(
                    symbol,
                    "tick without provider timestamp excluded from candles"
                )
            }
            TickDisposition::LateApplied => {
                self.record(
                    "atlsd_market_candle_corrections_total",
                    "Correction events for already published candles.",
                );
                tracing::warn!(
                    symbol,
                    "late tick merged into published candle; correction emitted"
                )
            }
            TickDisposition::Applied => {}
        }
        if let Some(candle) = correction {
            if let Err(err) = self.events.try_send(candle) {
                tracing::error!(error = %err, "candle event channel full; correction dropped");
            }
        }
    }
}

/// Freezes due candles once per second and keeps the correction window bounded.
pub async fn run_collector(handle: Arc<CandleEngineHandle>) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
    loop {
        interval.tick().await;
        let now = Utc::now();
        let events = {
            let mut engine = handle.engine.lock();
            let events = engine.collect_due(now);
            engine.evict_stale(5);
            events
        };
        for candle in events {
            if let Err(err) = handle.events.try_send(candle) {
                tracing::error!(error = %err, "candle event channel full; closed candle dropped");
            }
        }
    }
}

/// Publishes candle events to JetStream. A dropped publish is a delivery loss
/// only — the ClickHouse materialized view remains the storage source of truth
/// and clients catch up via history on reconnect.
pub async fn run_publisher(
    mut rx: tokio::sync::mpsc::Receiver<OhlcvCandle>,
    nats_url: &str,
    metrics: Option<Arc<atlsd_observability::MetricsRegistry>>,
) -> anyhow::Result<()> {
    let publisher = atlsd_eventbus::NatsPublisher::connect(nats_url).await?;
    tracing::info!(
        subject = atlsd_eventbus::subjects::MD_CANDLE_1M_V1,
        "candle event publisher connected"
    );

    while let Some(candle) = rx.recv().await {
        let msg_id = format!(
            "md.candle:{}:{}:{}",
            candle.symbol,
            candle.bucket_start.timestamp_millis(),
            candle.sequence
        );
        let payload = serde_json::to_string(&candle)?;
        match publisher
            .publish_str_with_id(atlsd_eventbus::subjects::MD_CANDLE_1M_V1, &payload, &msg_id)
            .await
        {
            Ok(()) => {
                if let Some(metrics) = &metrics {
                    metrics.inc(
                        "atlsd_market_candles_published_total",
                        "Candle events published to JetStream (closed + corrections).",
                    );
                }
            }
            Err(err) => {
                if let Some(metrics) = &metrics {
                    metrics.inc(
                        "atlsd_market_candle_publish_failures_total",
                        "Candle event publishes that failed (storage stays correct via the MV).",
                    );
                }
                tracing::error!(
                    error = %err,
                    symbol = %candle.symbol,
                    sequence = candle.sequence,
                    "candle publish failed; storage stays correct via ClickHouse MV"
                );
            }
        }
    }

    Ok(())
}

/// Rebuilds forming candles for the running minute after a restart so a
/// mid-minute crash does not lose the partial candle. Failure is tolerated:
/// storage correctness comes from the MV, this only restores event delivery.
pub async fn bootstrap_from_ticks(
    handle: &CandleEngineHandle,
    clickhouse: &crate::clickhouse::ClickHouseClient,
) {
    match clickhouse.recent_ticks(2).await {
        Ok(ticks) => {
            let now = Utc::now();
            let count = ticks.len();
            for tick in ticks {
                handle.accept_tick(
                    &tick.symbol,
                    Some(tick.ts_ms),
                    tick.price,
                    tick.volume,
                    "unknown",
                    now,
                );
            }
            tracing::info!(
                count,
                "candle engine bootstrapped from recent ClickHouse ticks"
            );
        }
        Err(err) => {
            tracing::warn!(error = %err, "candle bootstrap failed; forming candles start empty");
        }
    }
}

pub fn asset_class(asset_type: &str) -> AssetClass {
    match asset_type {
        "forex" => AssetClass::Forex,
        "stock" | "equity" => AssetClass::Equity,
        "index" => AssetClass::Index,
        "crypto" => AssetClass::Crypto,
        "commodity" => AssetClass::Commodity,
        "rates" => AssetClass::Rates,
        _ => AssetClass::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINUTE: i64 = 60_000;

    // Minute-aligned base so offset arithmetic in tests maps to clean buckets.
    const BASE_MS: i64 = 1_729_999_980_000;

    fn ts(offset_ms: i64) -> i64 {
        BASE_MS + offset_ms
    }

    fn engine() -> CandleEngine {
        CandleEngine::new(5)
    }

    fn now_after(offset_ms: i64) -> DateTime<Utc> {
        minute_bucket_from_ms(ts(offset_ms)) + chrono::Duration::seconds(120)
    }

    #[test]
    fn open_and_close_follow_provider_timestamp_not_arrival() {
        let mut engine = engine();
        let symbol = "BTCUSDT";
        // Arrive newest-first: close-priced tick arrives first, open-priced last.
        engine.accept_tick(
            symbol,
            Some(ts(50_000)),
            105.0,
            1.0,
            AssetClass::Crypto,
            now_after(0),
        );
        engine.accept_tick(
            symbol,
            Some(ts(20_000)),
            101.0,
            1.0,
            AssetClass::Crypto,
            now_after(0),
        );
        engine.accept_tick(
            symbol,
            Some(ts(10_000)),
            100.0,
            1.0,
            AssetClass::Crypto,
            now_after(0),
        );
        engine.accept_tick(
            symbol,
            Some(ts(40_000)),
            103.0,
            1.0,
            AssetClass::Crypto,
            now_after(0),
        );

        let events = engine.collect_due(now_after(60_000));
        assert_eq!(events.len(), 1);
        let candle = &events[0];
        assert_eq!(candle.open, 100.0, "open = price at smallest provider ts");
        assert_eq!(candle.close, 105.0, "close = price at largest provider ts");
        assert_eq!(candle.high, 105.0);
        assert_eq!(candle.low, 100.0);
        assert_eq!(candle.volume, 4.0);
        assert_eq!(candle.tick_count, 4);
        assert!(!candle.corrected);
        assert_eq!(candle.sequence, 1);
    }

    #[test]
    fn identical_ohlc_for_any_permutation_of_the_same_ticks() {
        let ticks = [
            (ts(0), 100.0),
            (ts(10_000), 102.5),
            (ts(20_000), 99.0),
            (ts(30_000), 101.0),
            (ts(40_000), 104.0),
            (ts(50_000), 100.5),
            (ts(59_999), 103.0),
        ];

        // Deterministic LCG shuffle over several seeds: OHLC must be identical.
        for seed in 1..8u64 {
            let mut order: Vec<usize> = (0..ticks.len()).collect();
            let mut state = seed;
            for i in (1..order.len()).rev() {
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                let j = (state >> 33) as usize % (i + 1);
                order.swap(i, j);
            }

            let mut engine = engine();
            for &i in &order {
                let (t, p) = ticks[i];
                engine.accept_tick(
                    "XAUUSD",
                    Some(t),
                    p,
                    2.0,
                    AssetClass::Commodity,
                    now_after(0),
                );
            }
            let events = engine.collect_due(now_after(60_000));
            assert_eq!(events.len(), 1, "seed {seed}");
            assert_eq!(events[0].open, 100.0, "seed {seed}");
            assert_eq!(events[0].close, 103.0, "seed {seed}");
            assert_eq!(events[0].high, 104.0, "seed {seed}");
            assert_eq!(events[0].low, 99.0, "seed {seed}");
            assert_eq!(events[0].volume, 14.0, "seed {seed}");
        }
    }

    #[test]
    fn ticks_without_provider_timestamp_are_excluded() {
        let mut engine = engine();
        let (disposition, event) =
            engine.accept_tick("EURUSD", None, 1.08, 1.0, AssetClass::Forex, now_after(0));
        assert_eq!(disposition, TickDisposition::NoTimestamp);
        assert!(event.is_none());
        assert!(engine.collect_due(now_after(60_000)).is_empty());
    }

    #[test]
    fn minute_boundaries_bucket_correctly() {
        assert_eq!(
            minute_bucket_from_ms(ts(59_999)),
            minute_bucket_from_ms(ts(0))
        );
        assert_ne!(
            minute_bucket_from_ms(ts(60_000)),
            minute_bucket_from_ms(ts(59_999))
        );
    }

    #[test]
    fn boundary_ticks_split_into_adjacent_minutes() {
        let mut engine = engine();
        engine.accept_tick(
            "BTCUSDT",
            Some(ts(59_999)),
            100.0,
            1.0,
            AssetClass::Crypto,
            now_after(0),
        );
        engine.accept_tick(
            "BTCUSDT",
            Some(ts(60_000)),
            200.0,
            1.0,
            AssetClass::Crypto,
            now_after(0),
        );

        let events = engine.collect_due(now_after(120_000));
        assert_eq!(events.len(), 2);
        assert_eq!(
            events[0].open, 100.0,
            "first minute only owns the 59.999 tick"
        );
        assert_eq!(events[0].close, 100.0);
        assert_eq!(events[1].open, 200.0, "00.000 tick opens the next minute");
    }

    #[test]
    fn late_tick_after_publish_emits_correction_with_higher_sequence() {
        let mut engine = engine();
        engine.accept_tick(
            "BTCUSDT",
            Some(ts(10_000)),
            100.0,
            1.0,
            AssetClass::Crypto,
            now_after(0),
        );
        engine.accept_tick(
            "BTCUSDT",
            Some(ts(50_000)),
            110.0,
            1.0,
            AssetClass::Crypto,
            now_after(0),
        );

        let closed = engine.collect_due(now_after(60_000));
        assert_eq!(closed.len(), 1);
        assert!(!closed[0].corrected);

        // A late tick arrives for the already published minute.
        let (disposition, correction) = engine.accept_tick(
            "BTCUSDT",
            Some(ts(55_000)),
            120.0,
            1.0,
            AssetClass::Crypto,
            now_after(70_000),
        );
        assert_eq!(disposition, TickDisposition::LateApplied);
        let correction = correction.expect("late tick must produce a correction event");
        assert!(correction.corrected);
        assert_eq!(correction.sequence, 2);
        assert_eq!(correction.high, 120.0, "correction merges into prior state");
        assert_eq!(correction.open, 100.0);
        assert_eq!(correction.tick_count, 3);

        // collect_due must not re-emit the corrected candle as a fresh close.
        assert!(engine.collect_due(now_after(80_000)).is_empty());
    }

    #[test]
    fn grace_window_holds_candle_back_until_elapsed() {
        let mut engine = CandleEngine::new(5);
        engine.accept_tick(
            "BTCUSDT",
            Some(ts(10_000)),
            100.0,
            1.0,
            AssetClass::Crypto,
            now_after(0),
        );

        // At minute end + 4s (still inside grace) nothing is due yet.
        let minute_end = minute_bucket_from_ms(ts(10_000)) + chrono::Duration::seconds(64);
        assert!(engine.collect_due(minute_end).is_empty());

        // At minute end + 5s+grace boundary the candle is due.
        let after_grace = minute_bucket_from_ms(ts(10_000)) + chrono::Duration::seconds(65);
        let events = engine.collect_due(after_grace);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn empty_minute_emits_no_candle() {
        let mut engine = engine();
        engine.accept_tick(
            "BTCUSDT",
            Some(ts(10_000)),
            100.0,
            1.0,
            AssetClass::Crypto,
            now_after(0),
        );
        let events = engine.collect_due(now_after(180_000));
        // Only one candle for one active minute; the empty minute is not a bar.
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn duplicate_cross_feed_ticks_both_counted_until_deduped_upstream() {
        // Cross-feed dedup happens upstream (JetStream Nats-Msg-Id at publish
        // and the latest-price monotonic guard); the engine itself is additive.
        let mut engine = engine();
        engine.accept_tick(
            "EURUSD",
            Some(ts(10_000)),
            1.08,
            1.0,
            AssetClass::Forex,
            now_after(0),
        );
        engine.accept_tick(
            "EURUSD",
            Some(ts(10_000)),
            1.08,
            1.0,
            AssetClass::Forex,
            now_after(0),
        );
        let events = engine.collect_due(now_after(60_000));
        assert_eq!(events[0].tick_count, 2);
    }

    #[test]
    fn evict_stale_bounds_the_correction_window() {
        let mut engine = engine();
        for minute in 0..10 {
            let base = minute * MINUTE;
            engine.accept_tick(
                "BTCUSDT",
                Some(ts(base + 10_000)),
                100.0 + minute as f64,
                1.0,
                AssetClass::Crypto,
                now_after(0),
            );
            engine.collect_due(now_after(base + 70_000));
        }

        engine.evict_stale(2);
        assert!(
            engine.candles.len() <= 3,
            "only recent minutes stay resident"
        );
    }

    #[test]
    fn collect_due_sorts_events_chronologically() {
        let mut engine = engine();
        engine.accept_tick(
            "AAA",
            Some(ts(120_000)),
            1.0,
            1.0,
            AssetClass::Unknown,
            now_after(0),
        );
        engine.accept_tick(
            "BBB",
            Some(ts(0)),
            1.0,
            1.0,
            AssetClass::Unknown,
            now_after(0),
        );
        let events = engine.collect_due(now_after(200_000));
        let minutes: Vec<DateTime<Utc>> = events.iter().map(|c| c.bucket_start).collect();
        let mut sorted = minutes.clone();
        sorted.sort();
        assert_eq!(minutes, sorted);
    }
}
