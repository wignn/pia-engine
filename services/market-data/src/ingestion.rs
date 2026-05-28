use atlsd_eventbus::{subjects, EventBusMode};
use chrono::{DateTime, TimeZone, Utc};
use futures_util::StreamExt;
use serde_json::Value;
use tracing::{debug, error, info, warn};

use crate::prices::{self, CachedPrice};
use crate::state::AppState;

const NATS_SUBJECTS: &[&str] = &[
    subjects::MD_RAW_PRIMARY_FX_QUOTES_V1,
    subjects::MD_RAW_SECONDARY_FX_QUOTES_V1,
    subjects::MD_RAW_CRYPTO_TRADES_V1,
    subjects::MD_RAW_INDEX_QUOTES_V1,
];

pub async fn run(state: AppState) {
    match EventBusMode::from_env_value(&state.config.eventbus_mode) {
        EventBusMode::Nats => run_nats(state).await,
        EventBusMode::Dual => {
            let redis_state = state.clone();
            tokio::spawn(async move {
                run_redis(redis_state).await;
            });
            run_nats(state).await;
        }
        EventBusMode::Redis => run_redis(state).await,
        EventBusMode::Noop => warn!("market-data ingestion disabled; EVENTBUS_MODE=noop"),
    }
}

async fn run_redis(state: AppState) {
    if !state.config.has_redis() {
        warn!("market-data Redis ingestion disabled; REDIS_URL is empty");
        return;
    }

    loop {
        if let Err(err) = subscribe_redis_loop(&state).await {
            error!(error = %err, "market-data Redis ingestion error, reconnecting in 5s");
        }
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}

async fn subscribe_redis_loop(state: &AppState) -> anyhow::Result<()> {
    let client = redis::Client::open(state.config.redis_url.clone())?;
    let mut pubsub = client.get_async_pubsub().await?;
    pubsub.psubscribe("ingestion:*").await?;
    info!("market-data connected to ingestion:* redis pubsub");

    while let Some(message) = pubsub.on_message().next().await {
        let payload: String = message.get_payload()?;
        handle_payload(&payload, state).await;
    }

    Ok(())
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
    let mut subscribers = futures_util::stream::SelectAll::new();
    for subject in NATS_SUBJECTS {
        subscribers.push(client.subscribe((*subject).to_string()).await?);
    }
    info!(subjects = ?NATS_SUBJECTS, "market-data connected to NATS ingestion subjects");

    while let Some(message) = subscribers.next().await {
        let payload = std::str::from_utf8(&message.payload)?;
        handle_payload(payload, state).await;
    }

    Ok(())
}

async fn handle_payload(payload: &str, state: &AppState) {
    let parsed: Value = match serde_json::from_str(payload) {
        Ok(value) => value,
        Err(err) => {
            warn!(error = %err, "failed to parse ingestion payload");
            return;
        }
    };

    let feed = parsed
        .get("feed")
        .and_then(|value| value.as_str())
        .unwrap_or("unknown");
    let raw_symbol = parsed
        .get("symbol")
        .and_then(|value| value.as_str())
        .unwrap_or("unknown");
    let price = parsed
        .get("price")
        .and_then(|value| value.as_f64())
        .unwrap_or(0.0);
    if price <= 0.0 {
        return;
    }

    let symbol = normalize_symbol(feed, raw_symbol);
    let asset_type = parsed
        .get("asset_type")
        .and_then(|value| value.as_str())
        .map(|value| value.to_lowercase())
        .unwrap_or_else(|| {
            match feed {
                "crypto" => "crypto",
                "primary_fx" | "secondary_fx" => "forex",
                "index" => "index",
                "stock" => "stock",
                _ => "unknown",
            }
            .to_string()
        });
    let received_at = parsed
        .get("received_at")
        .and_then(|value| value.as_str())
        .map(|value| value.to_string());

    state.prices.write().insert(
        symbol.clone(),
        CachedPrice {
            symbol: symbol.clone(),
            price,
            bid: parsed.get("bid").and_then(|value| value.as_f64()),
            ask: parsed.get("ask").and_then(|value| value.as_f64()),
            volume: parsed
                .get("volume")
                .or_else(|| parsed.get("quantity"))
                .and_then(|value| value.as_f64()),
            source: "market_data".to_string(),
            asset_type,
            received_at,
        },
    );

    let current = if state.config.write_latest {
        state.prices.read().get(&symbol).cloned()
    } else {
        None
    };
    if let Some(current) = current {
        let received_at = parse_received_at(current.received_at.as_deref());
        persist_latest_price(&state.db, &current, received_at).await;
        persist_clickhouse_price_tick(state, &current, received_at).await;
        let session = crate::session::session_status(
            &current.symbol,
            &current.asset_type,
            received_at,
            Some(&state.calendar),
        );
        if session.is_open {
            persist_ohlcv_candle(&state.db, &current, received_at).await;
            persist_clickhouse_ohlcv_candle(state, &current, received_at).await;
        } else {
            debug!(symbol = %current.symbol, state = %session.state, reason = %session.reason, "skipped ohlcv candle outside open session");
        }
    }
    debug!(symbol = %symbol, "updated market-data price cache");
}

async fn persist_clickhouse_price_tick(
    state: &AppState,
    price: &CachedPrice,
    received_at: DateTime<Utc>,
) {
    let Some(tx) = &state.tick_tx else {
        return;
    };

    if let Err(err) = tx.send((price.clone(), received_at)).await {
        warn!(error = %err, symbol = %price.symbol, "failed to enqueue ClickHouse price tick");
    }
}

async fn persist_clickhouse_ohlcv_candle(
    state: &AppState,
    price: &CachedPrice,
    received_at: DateTime<Utc>,
) {
    let Some(tx) = &state.candle_tx else {
        return;
    };
    let Some(minute) = minute_bucket(received_at) else {
        return;
    };

    if let Err(err) = tx.send((price.clone(), minute)).await {
        warn!(error = %err, symbol = %price.symbol, "failed to enqueue ClickHouse ohlcv candle");
    }
}

async fn persist_latest_price(
    pool: &sqlx::PgPool,
    price: &CachedPrice,
    received_at: DateTime<Utc>,
) {
    if let Err(err) = sqlx::query(
        "INSERT INTO market.market_latest_prices (symbol, price, bid, ask, volume, source, asset_type, received_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) ON CONFLICT (symbol) DO UPDATE SET price = EXCLUDED.price, bid = EXCLUDED.bid, ask = EXCLUDED.ask, volume = EXCLUDED.volume, source = EXCLUDED.source, asset_type = EXCLUDED.asset_type, received_at = EXCLUDED.received_at",
    )
    .bind(&price.symbol)
    .bind(price.price)
    .bind(price.bid)
    .bind(price.ask)
    .bind(price.volume.unwrap_or(0.0))
    .bind(&price.source)
    .bind(&price.asset_type)
    .bind(received_at)
    .execute(pool)
    .await
    {
        warn!(error = %err, symbol = %price.symbol, "failed to persist latest price");
    }
}

async fn persist_ohlcv_candle(
    pool: &sqlx::PgPool,
    price: &CachedPrice,
    received_at: DateTime<Utc>,
) {
    let Some(minute) = minute_bucket(received_at) else {
        return;
    };

    if let Err(err) = sqlx::query(
        "INSERT INTO market.ohlcv_candles (symbol, resolution, time, open, high, low, close, volume) VALUES ($1, '1m', $2, $3, $3, $3, $3, $4) ON CONFLICT (symbol, resolution, time) DO UPDATE SET high = GREATEST(market.ohlcv_candles.high, EXCLUDED.high), low = LEAST(market.ohlcv_candles.low, EXCLUDED.low), close = EXCLUDED.close, volume = market.ohlcv_candles.volume + EXCLUDED.volume",
    )
    .bind(&price.symbol)
    .bind(minute)
    .bind(price.price)
    .bind(price.volume.unwrap_or(0.0))
    .execute(pool)
    .await
    {
        warn!(error = %err, symbol = %price.symbol, "failed to persist ohlcv candle");
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
