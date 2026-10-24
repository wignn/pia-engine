use atlsd_contracts::{
    event_envelope::{stable_event_id, EventEnvelope},
    market::RawMarketTick,
};
use atlsd_eventbus::{
    nats::{dedup_headers, durable_pull_consumer_with_policy},
    subjects, EventBusMode,
};
use chrono::{DateTime, TimeZone, Utc};
use futures_util::StreamExt;
use tracing::{debug, error, info, warn};

use crate::prices::{self, CachedPrice};
use crate::state::AppState;

const NATS_SUBJECTS: &[&str] = &[
    subjects::MD_RAW_PRIMARY_FX_QUOTES_V1,
    subjects::MD_RAW_CRYPTO_TRADES_V1,
    subjects::MD_RAW_INDEX_QUOTES_V1,
];
const RAW_MARKET_CONSUMER: &str = "market-data-raw-v1";

/// Consumes durable raw ticks and acknowledges them only after processing succeeds.
pub async fn run(state: AppState) {
    match EventBusMode::from_env_value(&state.config.eventbus_mode) {
        EventBusMode::Nats | EventBusMode::Dual => run_nats(state).await,
        EventBusMode::Redis | EventBusMode::Noop => {
            warn!("market-data raw ticks require NATS JetStream; ingestion disabled for this eventbus mode")
        }
    }
}

async fn run_nats(state: AppState) {
    loop {
        if let Err(err) = subscribe_nats_loop(&state).await {
            error!(error = %err, "market-data NATS ingestion error, reconnecting in 5s");
        }
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}

async fn subscribe_nats_loop(state: &AppState) -> anyhow::Result<()> {
    let client = async_nats::connect(&state.config.nats_url).await?;
    atlsd_eventbus::nats::init_jetstream_streams(&client).await?;
    let jetstream = async_nats::jetstream::new(client.clone());
    let consumer = durable_pull_consumer_with_policy(
        &client,
        subjects::ATLSD_MARKET_STREAM,
        RAW_MARKET_CONSUMER,
        "md.raw.>",
        async_nats::jetstream::consumer::DeliverPolicy::All,
    )
    .await?;
    let mut messages = consumer.messages().await?;
    state
        .eventbus_ready
        .store(true, std::sync::atomic::Ordering::Release);
    let _readiness = ReadinessReset::new(state.eventbus_ready.clone());
    info!(
        consumer = RAW_MARKET_CONSUMER,
        subjects = ?NATS_SUBJECTS,
        "market-data consuming durable raw market events"
    );

    while let Some(result) = messages.next().await {
        let message = result?;
        let subject = message.message.subject.to_string();
        match std::str::from_utf8(&message.message.payload) {
            Ok(payload) => {
                match handle_payload(payload, state, Some((&jetstream, &subject))).await {
                    Ok(()) => {
                        if let Err(err) = message.ack().await {
                            warn!(error = %err, subject, "failed to acknowledge processed raw market event");
                        }
                    }
                    Err(err) => {
                        warn!(error = %err, subject, "raw market event processing failed; scheduling redelivery");
                        if let Err(ack_err) = message
                            .ack_with(async_nats::jetstream::AckKind::Nak(Some(
                                std::time::Duration::from_secs(3),
                            )))
                            .await
                        {
                            warn!(error = %ack_err, subject, "failed to nack raw market event");
                        }
                    }
                }
            }
            Err(err) => {
                warn!(error = %err, subject, "raw market event is not UTF-8; acknowledging poison event");
                let _ = message.ack().await;
            }
        }
    }

    Ok(())
}

struct ReadinessReset(std::sync::Arc<std::sync::atomic::AtomicBool>);

impl ReadinessReset {
    fn new(flag: std::sync::Arc<std::sync::atomic::AtomicBool>) -> Self {
        Self(flag)
    }
}

impl Drop for ReadinessReset {
    fn drop(&mut self) {
        self.0.store(false, std::sync::atomic::Ordering::Release);
    }
}

async fn handle_payload(
    payload: &str,
    state: &AppState,
    nats: Option<(&async_nats::jetstream::Context, &str)>,
) -> anyhow::Result<()> {
    let envelope = match decode_raw_market_event(payload) {
        Ok(value) => value,
        Err(err) => {
            warn!(error = %err, "failed to parse ingestion payload");
            return Ok(());
        }
    };
    let tick = envelope.payload.clone();
    let feed = tick.feed.as_str();
    let raw_symbol = tick.symbol.as_str();
    let price = tick.price;
    if price <= 0.0 {
        warn!(
            symbol = raw_symbol,
            price, "ignoring invalid raw market price"
        );
        return Ok(());
    }

    let symbol = normalize_symbol(feed, raw_symbol);
    let asset_type = tick
        .asset_type
        .as_deref()
        .map(str::to_lowercase)
        .unwrap_or_else(|| {
            match feed {
                "crypto" => "crypto",
                "primary_fx" => "forex",
                "index" => "index",
                "stock" => "stock",
                _ => "unknown",
            }
            .to_string()
        });
    let received_at = tick.received_at.clone();
    let timestamp_ms = tick.timestamp_ms.or(tick.trade_time_ms);
    let volume = tick.volume.or(tick.quantity);
    let existing_timestamp_ms = state
        .prices
        .read()
        .get(&symbol)
        .and_then(|price| price.timestamp_ms);
    if !should_accept_tick(existing_timestamp_ms, timestamp_ms) {
        debug!(
            symbol = %symbol,
            existing_timestamp_ms = ?existing_timestamp_ms,
            incoming_timestamp_ms = ?timestamp_ms,
            "ignored stale market-data tick"
        );
        return Ok(());
    }

    let current = CachedPrice {
        symbol: symbol.clone(),
        price,
        bid: tick.bid,
        ask: tick.ask,
        volume,
        source: tick.source,
        asset_type,
        received_at,
        timestamp_ms,
        feed: Some(feed.to_string()),
    };
    let received_at = parse_received_at(current.received_at.as_deref());
    let session = crate::session::session_status(
        &current.symbol,
        &current.asset_type,
        received_at,
        Some(&state.calendar),
    );

    if state.config.write_latest {
        persist_latest_price(&state.db, &current, received_at).await?;
    }
    if session.is_open {
        persist_clickhouse_price_tick(state, &current, received_at).await?;
        persist_ohlcv_candle(&state.db, &current, received_at).await?;
    } else {
        debug!(symbol = %current.symbol, state = %session.state, reason = %session.reason, "skipped tick and ohlcv candle outside open session");
    }

    if let Some((jetstream, subject)) = nats {
        let dedup_subject = match subject {
            subjects::MD_RAW_PRIMARY_FX_QUOTES_V1 => Some(subjects::MD_DEDUP_PRIMARY_FX_QUOTES_V1),
            subjects::MD_RAW_CRYPTO_TRADES_V1 => Some(subjects::MD_DEDUP_CRYPTO_TRADES_V1),
            subjects::MD_RAW_INDEX_QUOTES_V1 => Some(subjects::MD_DEDUP_INDEX_QUOTES_V1),
            _ => None,
        };
        if let Some(dedup_subject) = dedup_subject {
            let serialized = serde_json::to_vec(&envelope)?;
            jetstream
                .publish_with_headers(
                    dedup_subject.to_string(),
                    dedup_headers(&envelope.event_id.to_string()),
                    serialized.into(),
                )
                .await?
                .await?;
        }
    }

    if let Some(candle) = &state.candle {
        candle.accept_tick(
            &symbol,
            timestamp_ms,
            price,
            volume.unwrap_or(0.0),
            &current.asset_type,
            Utc::now(),
        );
    }

    state.prices.write().insert(symbol.clone(), current);
    debug!(symbol = %symbol, "updated market-data price cache");
    Ok(())
}

fn decode_raw_market_event(payload: &str) -> anyhow::Result<EventEnvelope<RawMarketTick>> {
    if let Ok(envelope) = serde_json::from_str::<EventEnvelope<RawMarketTick>>(payload) {
        return Ok(envelope);
    }

    let tick: RawMarketTick = serde_json::from_str(payload)?;
    let stable_key = format!(
        "{}:{}:{:?}:{:?}:{:?}",
        tick.feed,
        tick.symbol,
        tick.timestamp_ms.or(tick.trade_time_ms),
        tick.price,
        tick.volume.or(tick.quantity)
    );
    Ok(tick.into_envelope("md.raw.legacy.v1", stable_event_id(&stable_key)))
}

async fn persist_clickhouse_price_tick(
    state: &AppState,
    price: &CachedPrice,
    received_at: DateTime<Utc>,
) -> anyhow::Result<()> {
    let Some(tx) = &state.tick_tx else {
        return Ok(());
    };

    tx.send((price.clone(), received_at))
        .await
        .map_err(|err| anyhow::anyhow!("failed to enqueue ClickHouse price tick: {err}"))
}

async fn persist_latest_price(
    pool: &sqlx::PgPool,
    price: &CachedPrice,
    received_at: DateTime<Utc>,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO market.market_latest_prices (symbol, price, bid, ask, volume, source, asset_type, received_at, provider_ts_ms) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) ON CONFLICT (symbol) DO UPDATE SET price = EXCLUDED.price, bid = EXCLUDED.bid, ask = EXCLUDED.ask, volume = EXCLUDED.volume, source = EXCLUDED.source, asset_type = EXCLUDED.asset_type, received_at = EXCLUDED.received_at, provider_ts_ms = EXCLUDED.provider_ts_ms",
    )
    .bind(&price.symbol)
    .bind(price.price)
    .bind(price.bid)
    .bind(price.ask)
    .bind(price.volume.unwrap_or(0.0))
    .bind(&price.source)
    .bind(&price.asset_type)
    .bind(received_at)
    .bind(price.timestamp_ms)
    .execute(pool)
    .await?;
    Ok(())
}

async fn persist_ohlcv_candle(
    pool: &sqlx::PgPool,
    price: &CachedPrice,
    received_at: DateTime<Utc>,
) -> anyhow::Result<()> {
    let Some(minute) = minute_bucket(received_at) else {
        return Ok(());
    };

    sqlx::query(
        "INSERT INTO market.ohlcv_candles (symbol, resolution, time, open, high, low, close, volume) VALUES ($1, '1m', $2, $3, $3, $3, $3, $4) ON CONFLICT (symbol, resolution, time) DO UPDATE SET high = GREATEST(market.ohlcv_candles.high, EXCLUDED.high), low = LEAST(market.ohlcv_candles.low, EXCLUDED.low), close = EXCLUDED.close, volume = market.ohlcv_candles.volume + EXCLUDED.volume",
    )
    .bind(&price.symbol)
    .bind(minute)
    .bind(price.price)
    .bind(price.volume.unwrap_or(0.0))
    .execute(pool)
    .await?;
    Ok(())
}

fn should_accept_tick(
    existing_timestamp_ms: Option<i64>,
    incoming_timestamp_ms: Option<i64>,
) -> bool {
    match (existing_timestamp_ms, incoming_timestamp_ms) {
        (Some(existing), Some(incoming)) => incoming >= existing,
        _ => true,
    }
}

fn minute_bucket(received_at: DateTime<Utc>) -> Option<DateTime<Utc>> {
    Utc.timestamp_opt((received_at.timestamp() / 60) * 60, 0)
        .single()
}

fn parse_received_at(value: Option<&str>) -> DateTime<Utc> {
    value
        .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
        .map(|value| value.with_timezone(&Utc))
        .unwrap_or_else(Utc::now)
}

fn normalize_symbol(feed: &str, raw_symbol: &str) -> String {
    let symbol = raw_symbol.trim().to_uppercase();
    if feed == "crypto" {
        symbol.replace('-', "")
    } else {
        symbol.replace('/', "")
    }
}

#[allow(dead_code)]
pub async fn hydrate(state: &AppState) {
    match prices::hydrate_price_cache(state).await {
        Ok(count) => info!(count, "hydrated market-data price cache"),
        Err(err) => warn!(error = %err, "failed to hydrate market-data price cache"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn should_accept_tick_rejects_older_provider_timestamps_but_recovers_equal_ticks() {
        assert!(should_accept_tick(Some(100), Some(101)));
        assert!(should_accept_tick(Some(100), Some(100)));
        assert!(!should_accept_tick(Some(100), Some(99)));
    }
}
