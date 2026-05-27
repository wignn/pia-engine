use atlsd_eventbus::subjects;
use std::sync::Arc;

use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

use super::reconnect::ReconnectPolicy;
use crate::broker::BrokerPublisher;
use crate::config::Config;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct TradeEvent {
    #[serde(rename = "e")]
    event_type: String,
    #[serde(rename = "E")]
    event_time: i64,
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "p")]
    price: String,
    #[serde(rename = "q")]
    quantity: String,
    #[serde(rename = "T")]
    trade_time: i64,
    #[serde(rename = "m")]
    is_buyer_maker: bool,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct StreamMessage {
    stream: String,
    data: TradeEvent,
}

const WORKER: &str = "crypto_feed";
const FEED: &str = "crypto";
const SOURCE: &str = "market_data";
const TOPIC: &str = subjects::MD_RAW_CRYPTO_TRADES_V1;
const DEFAULT_SYMBOLS: &[&str] = &[
    "btcusdt", "ethusdt", "solusdt", "bnbusdt", "xrpusdt", "dogeusdt", "adausdt",
];

pub async fn run(cfg: Arc<Config>, broker: Arc<dyn BrokerPublisher>) {
    let mut backoff = ReconnectPolicy::new(cfg.reconnect_base_sec, cfg.reconnect_max_sec);

    let symbols = if cfg.crypto_symbols.is_empty() {
        DEFAULT_SYMBOLS
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
    } else {
        cfg.crypto_symbols.clone()
    };

    let streams: Vec<String> = symbols
        .iter()
        .map(|s| format!("{}@trade", s.to_lowercase()))
        .collect();

    let streams_param = streams.join("/");

    loop {
        if cfg.crypto_feed_ws_url.trim().is_empty() {
            error!(worker = WORKER, "crypto feed websocket URL not configured");
            tokio::time::sleep(backoff.next_delay()).await;
            continue;
        }
        let url = cfg
            .crypto_feed_ws_url
            .trim()
            .replace("{streams}", &streams_param);

        info!(worker = WORKER, symbols = ?symbols, streams = streams.len(), "connecting to market data websocket");

        let ws_stream = match connect_async(&url).await {
            Ok((stream, _response)) => {
                info!(worker = WORKER, "websocket connected");
                backoff.reset();
                stream
            }
            Err(e) => {
                let delay = backoff.next_delay();
                error!(worker = WORKER, error = %e, retry_secs = delay.as_secs(), "websocket connection failed");
                tokio::time::sleep(delay).await;
                continue;
            }
        };

        let (mut write, mut read) = ws_stream.split();
        let mut tick_count: u64 = 0;

        let disconnect_reason: &str;

        loop {
            match read.next().await {
                Some(Ok(Message::Text(text))) => {
                    if let Err(e) = handle_message(&text, &*broker, &mut tick_count).await {
                        debug!(worker = WORKER, error = %e, "message handling error");
                    }
                }
                Some(Ok(Message::Ping(data))) => {
                    if let Err(e) = write.send(Message::Pong(data)).await {
                        warn!(worker = WORKER, error = %e, "pong send failed");
                    }
                }
                Some(Ok(Message::Close(_))) => {
                    info!(worker = WORKER, "server sent close frame");
                    disconnect_reason = "server_close";
                    break;
                }
                Some(Err(e)) => {
                    error!(worker = WORKER, error = %e, "websocket read error");
                    disconnect_reason = "read_error";
                    break;
                }
                None => {
                    info!(worker = WORKER, "websocket stream ended");
                    disconnect_reason = "stream_end";
                    break;
                }
                _ => {}
            }
        }

        info!(
            worker = WORKER,
            reason = disconnect_reason,
            ticks = tick_count,
            "disconnecting websocket"
        );

        let _ = write.send(Message::Close(None)).await;
        let _ = write.close().await;

        let delay = backoff.next_delay();
        warn!(
            worker = WORKER,
            retry_secs = delay.as_secs(),
            "reconnecting after disconnect"
        );
        tokio::time::sleep(delay).await;
    }
}

async fn handle_message(
    text: &str,
    broker: &dyn BrokerPublisher,
    tick_count: &mut u64,
) -> anyhow::Result<()> {
    let msg: StreamMessage = serde_json::from_str(text)?;

    if msg.data.event_type != "trade" {
        return Ok(());
    }

    let price: f64 = msg.data.price.parse().unwrap_or(0.0);
    let quantity: f64 = msg.data.quantity.parse().unwrap_or(0.0);

    if price <= 0.0 {
        return Ok(());
    }

    *tick_count += 1;

    let payload = json!({
        "feed": FEED,
        "source": SOURCE,
        "symbol": &msg.data.symbol,
        "price": price,
        "quantity": quantity,
        "trade_time_ms": msg.data.trade_time,
        "is_buyer_maker": msg.data.is_buyer_maker,
        "received_at": Utc::now().to_rfc3339(),
    });

    let payload_str = payload.to_string();
    if let Err(e) = broker.publish(TOPIC, &payload_str).await {
        warn!(worker = WORKER, error = %e, symbol = %msg.data.symbol, "broker publish failed");
    }

    Ok(())
}
