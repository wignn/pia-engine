use chrono::Timelike;
use futures_util::StreamExt;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, error, info, warn};

use crate::ws;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct CachedPrice {
    pub symbol: String,
    pub price: f64,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub volume: Option<f64>,
    pub source: String,
    pub asset_type: String,
    pub received_at: Option<String>,
    #[serde(skip)]
    pub updated_at: Option<Instant>,
}

#[derive(Debug, Clone)]
struct CandleState {
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    minute_timestamp: chrono::DateTime<chrono::Utc>,
}

pub static PRICE_CACHE: Lazy<Arc<RwLock<HashMap<String, CachedPrice>>>> =
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

pub fn get_all_prices() -> Vec<CachedPrice> {
    PRICE_CACHE.read().values().cloned().collect()
}

pub fn get_price(symbol: &str) -> Option<CachedPrice> {
    PRICE_CACHE.read().get(&symbol.to_uppercase()).cloned()
}

pub async fn run(redis_url: String, hub: Arc<ws::Hub>, pool: sqlx::PgPool) {
    loop {
        if let Err(e) = subscribe_loop(&redis_url, &hub, &pool).await {
            error!(error = %e, "ingestion subscriber error, reconnecting in 5s");
        }
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}

async fn subscribe_loop(
    redis_url: &str,
    hub: &Arc<ws::Hub>,
    pool: &sqlx::PgPool,
) -> anyhow::Result<()> {
    let client = redis::Client::open(redis_url)?;
    let mut pubsub = client.get_async_pubsub().await?;

    pubsub.psubscribe("ingestion:*").await?;
    info!("ingestion subscriber connected, listening on ingestion:*");

    let mut active_candles: HashMap<String, CandleState> = HashMap::new();

    loop {
        let msg = pubsub.on_message().next().await;

        let msg = match msg {
            Some(m) => m,
            None => {
                warn!("ingestion pubsub stream ended");
                return Ok(());
            }
        };

        let channel: String = msg.get_channel()?;
        let payload: String = msg.get_payload()?;

        debug!(channel = %channel, payload_len = payload.len(), "ingestion message received");

        let parsed: Value = match serde_json::from_str(&payload) {
            Ok(v) => v,
            Err(e) => {
                warn!(error = %e, channel = %channel, "failed to parse ingestion payload");
                continue;
            }
        };

        let source = parsed
            .get("source")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let raw_symbol = parsed
            .get("symbol")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let price = parsed.get("price").and_then(|v| v.as_f64()).unwrap_or(0.0);

        if price <= 0.0 {
            continue;
        }

        let symbol = normalize_symbol(source, raw_symbol);
        let bid = parsed.get("bid").and_then(|v| v.as_f64());
        let ask = parsed.get("ask").and_then(|v| v.as_f64());
        let volume = parsed.get("volume").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let received_at = parsed
            .get("received_at")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let asset_type = match source {
            "binance" => "crypto",
            "finnhub" | "tiingo" => "forex",
            "yahoo" => "index",
            _ => "unknown",
        }
        .to_string();

        {
            let mut cache = PRICE_CACHE.write();
            cache.insert(
                symbol.clone(),
                CachedPrice {
                    symbol: symbol.clone(),
                    price,
                    bid,
                    ask,
                    volume: Some(volume),
                    source: source.to_string(),
                    asset_type: asset_type.clone(),
                    received_at: received_at.clone(),
                    updated_at: Some(Instant::now()),
                },
            );
        }

        let tick_time = parse_received_at(received_at.as_deref());
        let tick_min = truncate_to_minute(tick_time);

        let mut completed_candle = None;

        if let Some(candle) = active_candles.get_mut(&symbol) {
            if tick_min == candle.minute_timestamp {
                candle.high = candle.high.max(price);
                candle.low = candle.low.min(price);
                candle.close = price;
                candle.volume += volume;
            } else if tick_min > candle.minute_timestamp {
                completed_candle = Some(candle.clone());

                *candle = CandleState {
                    open: price,
                    high: price,
                    low: price,
                    close: price,
                    volume,
                    minute_timestamp: tick_min,
                };
            }
        } else {
            active_candles.insert(
                symbol.clone(),
                CandleState {
                    open: price,
                    high: price,
                    low: price,
                    close: price,
                    volume,
                    minute_timestamp: tick_min,
                },
            );
        }

        if let Some(candle) = completed_candle {
            let pool_clone = pool.clone();
            let symbol_clone = symbol.clone();
            tokio::spawn(async move {
                let res = sqlx::query(
                    "INSERT INTO ohlcv_candles (symbol, resolution, time, open, high, low, close, volume) \
                     VALUES ($1, '1m', $2, $3, $4, $5, $6, $7) \
                     ON CONFLICT (symbol, resolution, time) DO UPDATE SET \
                     high = EXCLUDED.high, low = EXCLUDED.low, close = EXCLUDED.close, volume = EXCLUDED.volume"
                )
                .bind(&symbol_clone)
                .bind(candle.minute_timestamp)
                .bind(candle.open)
                .bind(candle.high)
                .bind(candle.low)
                .bind(candle.close)
                .bind(candle.volume)
                .execute(&pool_clone)
                .await;

                if let Err(e) = res {
                    error!(error = %e, symbol = %symbol_clone, "failed to save candle to database");
                } else {
                    debug!(symbol = %symbol_clone, time = %candle.minute_timestamp, "saved completed 1m candle to timeseries database");
                }
            });
        }

        let tick_data = serde_json::json!({
            "tick": {
                "symbol": &symbol,
                "price": price,
                "bid": bid,
                "ask": ask,
                "volume": Some(volume),
                "source": source,
                "asset_type": &asset_type,
                "received_at": received_at,
            },
            "asset_type": &asset_type,
        });

        let _ = hub
            .broadcast(ws::EVENT_MARKET_TRADE, tick_data, "market_data")
            .await;
    }
}

fn parse_received_at(s: Option<&str>) -> chrono::DateTime<chrono::Utc> {
    s.and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(chrono::Utc::now)
}

fn truncate_to_minute(dt: chrono::DateTime<chrono::Utc>) -> chrono::DateTime<chrono::Utc> {
    dt.with_second(0)
        .and_then(|t| t.with_nanosecond(0))
        .unwrap_or(dt)
}

fn normalize_symbol(source: &str, raw: &str) -> String {
    match source {
        "finnhub" => {
            if let Some((_venue, pair)) = raw.split_once(':') {
                pair.replace('_', "").to_uppercase()
            } else {
                raw.replace('_', "").to_uppercase()
            }
        }
        "tiingo" => raw.to_uppercase(),
        "binance" => {
            if let Some((_prefix, pair)) = raw.to_uppercase().split_once(':') {
                pair.to_string()
            } else {
                raw.to_uppercase()
            }
        }
        _ => raw.to_uppercase(),
    }
}
