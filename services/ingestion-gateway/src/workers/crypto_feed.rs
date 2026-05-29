use atlsd_eventbus::subjects;
use std::{sync::Arc, time::Duration};

use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

use super::{
    publish_queue::{PublishEvent, PublishQueue, enqueue_or_drop, spawn_publisher},
    reconnect::ReconnectPolicy,
};
use crate::broker::BrokerPublisher;
use crate::config::Config;
use crate::health::HealthRegistry;

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

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum BinanceTradeMessage {
    Combined(StreamMessage),
    Raw(TradeEvent),
}

impl BinanceTradeMessage {
    fn into_trade(self) -> TradeEvent {
        match self {
            Self::Combined(message) => message.data,
            Self::Raw(event) => event,
        }
    }
}

const WORKER: &str = "crypto_feed";
const FEED: &str = "crypto";
const SOURCE: &str = "market_data";
const TOPIC: &str = subjects::MD_RAW_CRYPTO_TRADES_V1;
const PUBLISH_TIMEOUT: Duration = Duration::from_secs(5);
const READ_IDLE_TIMEOUT: Duration = Duration::from_secs(90);
const PUBLISH_QUEUE_CAPACITY: usize = 50_000;
const PROGRESS_LOG_INTERVAL: u64 = 10_000;
const DEFAULT_SYMBOLS: &[&str] = &[
    "btcusdt", "ethusdt", "solusdt", "bnbusdt", "xrpusdt", "dogeusdt", "adausdt",
];

pub async fn run(cfg: Arc<Config>, broker: Arc<dyn BrokerPublisher>, health: HealthRegistry) {
    let mut backoff = ReconnectPolicy::new(cfg.reconnect_base_sec, cfg.reconnect_max_sec);
    let publish_queue = spawn_publisher(
        WORKER,
        broker,
        health.clone(),
        PUBLISH_QUEUE_CAPACITY,
        PUBLISH_TIMEOUT,
        PROGRESS_LOG_INTERVAL,
    );

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
                health.set_connected(WORKER, true).await;
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
        let mut queued_count: u64 = 0;

        let disconnect_reason: &str;

        loop {
            match tokio::time::timeout(READ_IDLE_TIMEOUT, read.next()).await {
                Ok(Some(Ok(Message::Text(text)))) => {
                    if let Err(e) = handle_message(
                        &text,
                        &publish_queue,
                        &health,
                        &mut tick_count,
                        &mut queued_count,
                    ) {
                        debug!(worker = WORKER, error = %e, "message handling error");
                    }
                }
                Ok(Some(Ok(Message::Ping(data)))) => {
                    if let Err(e) = write.send(Message::Pong(data)).await {
                        warn!(worker = WORKER, error = %e, "pong send failed");
                    }
                }
                Ok(Some(Ok(Message::Close(_)))) => {
                    info!(worker = WORKER, "server sent close frame");
                    disconnect_reason = "server_close";
                    break;
                }
                Ok(Some(Err(e))) => {
                    error!(worker = WORKER, error = %e, "websocket read error");
                    disconnect_reason = "read_error";
                    break;
                }
                Ok(None) => {
                    info!(worker = WORKER, "websocket stream ended");
                    disconnect_reason = "stream_end";
                    break;
                }
                Ok(Some(Ok(_))) => {}
                Err(_) => {
                    warn!(
                        worker = WORKER,
                        idle_secs = READ_IDLE_TIMEOUT.as_secs(),
                        ticks = tick_count,
                        "websocket read idle timeout"
                    );
                    disconnect_reason = "read_idle_timeout";
                    break;
                }
            }
        }

        info!(
            worker = WORKER,
            reason = disconnect_reason,
            ticks = tick_count,
            queued = queued_count,
            "disconnecting websocket"
        );
        health.record_disconnect(WORKER, disconnect_reason).await;

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

fn handle_message(
    text: &str,
    publish_queue: &PublishQueue,
    health: &HealthRegistry,
    tick_count: &mut u64,
    queued_count: &mut u64,
) -> anyhow::Result<()> {
    let event = parse_trade_message(text)?;

    if event.event_type != "trade" {
        return Ok(());
    }

    let price: f64 = event.price.parse().unwrap_or(0.0);
    let quantity: f64 = event.quantity.parse().unwrap_or(0.0);

    if price <= 0.0 {
        return Ok(());
    }

    *tick_count += 1;
    let health = health.clone();

    let payload = json!({
        "feed": FEED,
        "source": SOURCE,
        "symbol": &event.symbol,
        "price": price,
        "quantity": quantity,
        "trade_time_ms": event.trade_time,
        "is_buyer_maker": event.is_buyer_maker,
        "received_at": Utc::now().to_rfc3339(),
    });

    let queued = enqueue_or_drop(
        WORKER,
        publish_queue,
        PublishEvent {
            subject: TOPIC,
            payload: payload.to_string(),
            symbol: event.symbol,
        },
        queued_count,
    );
    let queue_depth = PUBLISH_QUEUE_CAPACITY.saturating_sub(publish_queue.capacity());

    tokio::spawn(async move {
        health.record_tick(WORKER).await;
        if queued {
            health.record_queued(WORKER, queue_depth).await;
        } else {
            health.record_drop(WORKER, queue_depth).await;
        }
    });

    Ok(())
}

fn parse_trade_message(text: &str) -> anyhow::Result<TradeEvent> {
    Ok(serde_json::from_str::<BinanceTradeMessage>(text)?.into_trade())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_combined_stream_trade_payload() {
        let trade = parse_trade_message(
            r#"{"stream":"btcusdt@trade","data":{"e":"trade","E":1779940191742,"s":"BTCUSDT","t":6328956968,"p":"73409.99000000","q":"0.00007000","T":1779940191742,"m":true,"M":true}}"#,
        )
        .unwrap();

        assert_eq!(trade.symbol, "BTCUSDT");
        assert_eq!(trade.price, "73409.99000000");
        assert_eq!(trade.quantity, "0.00007000");
    }

    #[test]
    fn parses_multiple_combined_stream_symbols() {
        let eth = parse_trade_message(
            r#"{"stream":"ethusdt@trade","data":{"e":"trade","E":1779941620916,"s":"ETHUSDT","t":4049338705,"p":"1979.94000000","q":"0.00510000","T":1779941620915,"m":true,"M":true}}"#,
        )
        .unwrap();
        let btc = parse_trade_message(
            r#"{"stream":"btcusdt@trade","data":{"e":"trade","E":1779941620961,"s":"BTCUSDT","t":6329162610,"p":"73109.38000000","q":"0.00040000","T":1779941620961,"m":false,"M":true}}"#,
        )
        .unwrap();

        assert_eq!(eth.symbol, "ETHUSDT");
        assert_eq!(eth.price, "1979.94000000");
        assert_eq!(eth.quantity, "0.00510000");
        assert_eq!(btc.symbol, "BTCUSDT");
        assert_eq!(btc.price, "73109.38000000");
        assert_eq!(btc.quantity, "0.00040000");
    }

    #[test]
    fn parses_raw_trade_payload() {
        let trade = parse_trade_message(
            r#"{"e":"trade","E":1779940191742,"s":"BTCUSDT","t":6328956968,"p":"73409.99000000","q":"0.00007000","T":1779940191742,"m":true,"M":true}"#,
        )
        .unwrap();

        assert_eq!(trade.symbol, "BTCUSDT");
        assert_eq!(trade.price, "73409.99000000");
        assert_eq!(trade.quantity, "0.00007000");
    }
}
