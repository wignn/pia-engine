use std::sync::Arc;

use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

use crate::broker::BrokerPublisher;
use crate::config::Config;
use super::reconnect::ReconnectPolicy;

#[derive(Debug, Deserialize)]
struct BinanceTrade {
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
struct BinanceStreamMessage {
    stream: String,
    data: BinanceTrade,
}

const TOPIC: &str = "binance:crypto";

const DEFAULT_SYMBOLS: &[&str] = &[
    "btcusdt",
    "ethusdt",
    "solusdt",
    "bnbusdt",
    "xrpusdt",
    "dogeusdt",
    "adausdt",
];

pub async fn run(cfg: Arc<Config>, broker: Arc<dyn BrokerPublisher>) {
    let mut backoff = ReconnectPolicy::new(cfg.reconnect_base_sec, cfg.reconnect_max_sec);

    let symbols = if cfg.binance_symbols.is_empty() {
        DEFAULT_SYMBOLS.iter().map(|s| s.to_string()).collect::<Vec<_>>()
    } else {
        cfg.binance_symbols.clone()
    };

    let streams: Vec<String> = symbols
        .iter()
        .map(|s| format!("{}@trade", s.to_lowercase()))
        .collect();

    let streams_param = streams.join("/");

    loop {
        let url = format!(
            "wss://stream.binance.com:9443/stream?streams={}",
            streams_param
        );

        info!(
            worker = "binance",
            symbols = ?symbols,
            streams = streams.len(),
            "connecting to binance websocket"
        );

        let ws_stream = match connect_async(&url).await {
            Ok((stream, _response)) => {
                info!(worker = "binance", "websocket connected");
                backoff.reset();
                stream
            }
            Err(e) => {
                let delay = backoff.next_delay();
                error!(
                    worker = "binance",
                    error = %e,
                    retry_secs = delay.as_secs(),
                    "websocket connection failed"
                );
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
                        debug!(worker = "binance", error = %e, "message handling error");
                    }
                }
                Some(Ok(Message::Ping(data))) => {
                    if let Err(e) = write.send(Message::Pong(data)).await {
                        warn!(worker = "binance", error = %e, "pong send failed");
                    }
                }
                Some(Ok(Message::Close(_))) => {
                    info!(worker = "binance", "server sent close frame");
                    disconnect_reason = "server_close";
                    break;
                }
                Some(Err(e)) => {
                    error!(worker = "binance", error = %e, "websocket read error");
                    disconnect_reason = "read_error";
                    break;
                }
                None => {
                    info!(worker = "binance", "websocket stream ended");
                    disconnect_reason = "stream_end";
                    break;
                }
                _ => {}
            }
        }

        info!(
            worker = "binance",
            reason = disconnect_reason,
            ticks = tick_count,
            "disconnecting websocket"
        );

        let _ = write.send(Message::Close(None)).await;
        let _ = write.close().await;

        let delay = backoff.next_delay();
        warn!(
            worker = "binance",
            retry_secs = delay.as_secs(),
            "reconnecting after disconnect"
        );
        tokio::time::sleep(delay).await;
    }
}

/// Parse and publish a single Binance combined stream message.
async fn handle_message(
    text: &str,
    broker: &dyn BrokerPublisher,
    tick_count: &mut u64,
) -> anyhow::Result<()> {
    let msg: BinanceStreamMessage = serde_json::from_str(text)?;

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
        "source": "binance",
        "symbol": &msg.data.symbol,
        "price": price,
        "quantity": quantity,
        "trade_time_ms": msg.data.trade_time,
        "is_buyer_maker": msg.data.is_buyer_maker,
        "received_at": Utc::now().to_rfc3339(),
    });

    let payload_str = payload.to_string();
    if let Err(e) = broker.publish(TOPIC, &payload_str).await {
        warn!(
            worker = "binance",
            error = %e,
            symbol = %msg.data.symbol,
            "broker publish failed"
        );
    }

    Ok(())
}
