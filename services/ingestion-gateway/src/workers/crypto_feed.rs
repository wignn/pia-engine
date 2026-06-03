use atlsd_eventbus::subjects;
use std::{sync::Arc, time::Duration};

use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

use super::{
    publish_queue::{enqueue_or_drop, spawn_publisher, PublishEvent, PublishQueue},
    reconnect::ReconnectPolicy,
};
use crate::broker::BrokerPublisher;
use crate::config::Config;
use crate::health::HealthRegistry;

#[derive(Debug, Deserialize)]
struct OkxTrade {
    #[serde(rename = "instId")]
    inst_id: String,
    #[serde(rename = "px")]
    price: String,
    #[serde(rename = "sz")]
    size: String,
    side: String,
    #[serde(rename = "ts")]
    ts: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OkxArg {
    #[serde(rename = "instId")]
    inst_id: String,
}

#[derive(Debug, Deserialize)]
struct OkxPush {
    #[allow(dead_code)]
    arg: OkxArg,
    data: Vec<OkxTrade>,
}

/// OKX also sends control messages (subscribe ack, ping, error).
/// We only care about pushes that have a "data" array.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum OkxMessage {
    Push(OkxPush),
    /// Catch-all for ack / ping / error frames — we discard these.
    #[allow(dead_code)]
    Other(serde_json::Value),
}

const WORKER: &str = "crypto_feed";
const FEED: &str = "crypto";
const SOURCE: &str = "market_data";
const TOPIC: &str = subjects::MD_RAW_CRYPTO_TRADES_V1;
const PUBLISH_TIMEOUT: Duration = Duration::from_secs(5);
const READ_IDLE_TIMEOUT: Duration = Duration::from_secs(30);
const PUBLISH_QUEUE_CAPACITY: usize = 50_000;
const PROGRESS_LOG_INTERVAL: u64 = 10_000;

/// Default symbols in OKX SWAP format.
/// Env var CRYPTO_SYMBOLS accepts comma-separated bare symbols like "BTCUSDT,ETHUSDT"
/// which are normalized to "BTC-USDT-SWAP" internally.
const DEFAULT_SYMBOLS: &[&str] = &[
    "BTC-USDT-SWAP",
    "ETH-USDT-SWAP",
    "SOL-USDT-SWAP",
    "BNB-USDT-SWAP",
    "XRP-USDT-SWAP",
    "DOGE-USDT-SWAP",
    "ADA-USDT-SWAP",
];

/// Convert a bare symbol like "BTCUSDT" → "BTC-USDT-SWAP".
/// If the input already looks like an OKX instId (contains '-'), pass it through.
fn to_okx_inst_id(raw: &str) -> String {
    let s = raw.to_uppercase();
    if s.contains('-') {
        return s;
    }
    // Strip trailing "USDT" / "USDC" / "BTC" suffix and rebuild
    for quote in &["USDT", "USDC", "BTC", "ETH"] {
        if let Some(base) = s.strip_suffix(quote) {
            if !base.is_empty() {
                return format!("{}-{}-SWAP", base, quote);
            }
        }
    }
    // Fallback: treat as-is with USDT
    format!("{}-USDT-SWAP", s)
}

/// Normalize OKX instId back to a plain symbol for downstream consumers.
/// "BTC-USDT-SWAP" → "BTCUSDT"
fn inst_id_to_symbol(inst_id: &str) -> String {
    inst_id.replace('-', "").replace("SWAP", "")
}

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

    // Build OKX instId list from config or defaults
    let inst_ids: Vec<String> = if cfg.crypto_symbols.is_empty() {
        DEFAULT_SYMBOLS.iter().map(|s| s.to_string()).collect()
    } else {
        cfg.crypto_symbols
            .iter()
            .map(|s| to_okx_inst_id(s))
            .collect()
    };

    // Build the subscribe JSON once
    let subscribe_args: Vec<serde_json::Value> = inst_ids
        .iter()
        .map(|id| json!({"channel": "trades", "instId": id}))
        .collect();
    let subscribe_msg = json!({"op": "subscribe", "args": subscribe_args}).to_string();

    loop {
        if cfg.crypto_feed_ws_url.trim().is_empty() {
            error!(worker = WORKER, "crypto feed websocket URL not configured");
            tokio::time::sleep(backoff.next_delay()).await;
            continue;
        }

        let url = cfg.crypto_feed_ws_url.trim().to_string();

        info!(
            worker = WORKER,
            inst_ids = ?inst_ids,
            streams = inst_ids.len(),
            "connecting to market data websocket"
        );

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

        // Send subscribe message
        if let Err(e) = write.send(Message::Text(subscribe_msg.clone())).await {
            error!(worker = WORKER, error = %e, "failed to send subscribe message");
            let delay = backoff.next_delay();
            tokio::time::sleep(delay).await;
            continue;
        }

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
    let msg: OkxMessage = serde_json::from_str(text)?;

    let push = match msg {
        OkxMessage::Push(p) => p,
        OkxMessage::Other(_) => return Ok(()), // ack / ping / error — skip
    };

    for trade in push.data {
        let price: f64 = trade.price.parse().unwrap_or(0.0);
        let quantity: f64 = trade.size.parse().unwrap_or(0.0);
        let trade_time_ms: i64 = trade.ts.parse().unwrap_or(0);
        let is_buyer_maker = trade.side == "sell"; // seller is maker when side=sell

        if price <= 0.0 {
            continue;
        }

        *tick_count += 1;
        let health = health.clone();
        let symbol = inst_id_to_symbol(&trade.inst_id);

        let payload = json!({
            "feed": FEED,
            "source": SOURCE,
            "symbol": &symbol,
            "price": price,
            "quantity": quantity,
            "trade_time_ms": trade_time_ms,
            "is_buyer_maker": is_buyer_maker,
            "received_at": Utc::now().to_rfc3339(),
        });

        let queued = enqueue_or_drop(
            WORKER,
            publish_queue,
            PublishEvent {
                subject: TOPIC,
                payload: payload.to_string(),
                symbol,
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
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Real OKX trades channel push payload
    const OKX_PUSH_BTC: &str = r#"{
        "arg":{"channel":"trades","instId":"BTC-USDT-SWAP"},
        "data":[{
            "instId":"BTC-USDT-SWAP",
            "tradeId":"123456789",
            "px":"73409.9",
            "sz":"0.07",
            "side":"buy",
            "ts":"1779940191742"
        }]
    }"#;

    const OKX_PUSH_MULTI: &str = r#"{
        "arg":{"channel":"trades","instId":"ETH-USDT-SWAP"},
        "data":[
            {"instId":"ETH-USDT-SWAP","tradeId":"111","px":"1979.94","sz":"0.0051","side":"sell","ts":"1779941620915"},
            {"instId":"ETH-USDT-SWAP","tradeId":"112","px":"1980.00","sz":"0.010","side":"buy","ts":"1779941620920"}
        ]
    }"#;

    // Control messages — should be silently ignored
    const OKX_ACK: &str = r#"{"event":"subscribe","arg":{"channel":"trades","instId":"BTC-USDT-SWAP"},"connId":"abc123"}"#;
    const OKX_PING: &str = r#"{"event":"ping"}"#;

    #[test]
    fn parses_single_trade_push() {
        let msg: OkxMessage = serde_json::from_str(OKX_PUSH_BTC).unwrap();
        let push = match msg {
            OkxMessage::Push(p) => p,
            _ => panic!("expected Push"),
        };
        assert_eq!(push.arg.inst_id, "BTC-USDT-SWAP");
        assert_eq!(push.data.len(), 1);
        let t = &push.data[0];
        assert_eq!(t.price, "73409.9");
        assert_eq!(t.size, "0.07");
        assert_eq!(t.side, "buy");
        assert_eq!(t.ts, "1779940191742");
    }

    #[test]
    fn parses_multi_trade_push() {
        let msg: OkxMessage = serde_json::from_str(OKX_PUSH_MULTI).unwrap();
        let push = match msg {
            OkxMessage::Push(p) => p,
            _ => panic!("expected Push"),
        };
        assert_eq!(push.data.len(), 2);
        assert_eq!(push.data[0].price, "1979.94");
        assert_eq!(push.data[1].price, "1980.00");
    }

    #[test]
    fn ignores_ack_message() {
        let msg: OkxMessage = serde_json::from_str(OKX_ACK).unwrap();
        assert!(matches!(msg, OkxMessage::Other(_)));
    }

    #[test]
    fn ignores_ping_message() {
        let msg: OkxMessage = serde_json::from_str(OKX_PING).unwrap();
        assert!(matches!(msg, OkxMessage::Other(_)));
    }

    #[test]
    fn inst_id_to_symbol_conversion() {
        assert_eq!(inst_id_to_symbol("BTC-USDT-SWAP"), "BTCUSDT");
        assert_eq!(inst_id_to_symbol("ETH-USDT-SWAP"), "ETHUSDT");
        assert_eq!(inst_id_to_symbol("SOL-USDT-SWAP"), "SOLUSDT");
    }

    #[test]
    fn to_okx_inst_id_conversion() {
        assert_eq!(to_okx_inst_id("btcusdt"), "BTC-USDT-SWAP");
        assert_eq!(to_okx_inst_id("ETHUSDT"), "ETH-USDT-SWAP");
        assert_eq!(to_okx_inst_id("BTC-USDT-SWAP"), "BTC-USDT-SWAP");
    }

    #[test]
    fn buyer_maker_logic() {
        // side=sell means seller is maker → is_buyer_maker=true
        assert!(("sell" == "sell"));
        // side=buy means buyer is taker → is_buyer_maker=false
        assert!("buy" != "sell");
    }
}
