use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::time::{interval, MissedTickBehavior};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

use crate::broker::BrokerPublisher;
use crate::config::Config;
use crate::market_hours;
use super::reconnect::ReconnectPolicy;

const TICKERS: &[&str] = &["gbpusd", "eurusd", "usdjpy"];
const TOPIC: &str = "tiingo:forex";
const WSS_URL: &str = "wss://api.tiingo.com/fx";

pub async fn run(cfg: Arc<Config>, broker: Arc<dyn BrokerPublisher>) {
    let mut backoff = ReconnectPolicy::new(cfg.reconnect_base_sec, cfg.reconnect_max_sec);

    loop {
        if !market_hours::is_market_open() {
            let wait = market_hours::duration_until_next_open();
            info!(
                worker = "tiingo",
                wait_secs = wait.as_secs(),
                "forex market closed, sleeping until open"
            );
            tokio::time::sleep(wait).await;
            continue;
        }

        info!(
            worker = "tiingo",
            tickers = ?TICKERS,
            "connecting to tiingo forex websocket"
        );

        let ws_stream = match connect_async(WSS_URL).await {
            Ok((stream, _response)) => {
                info!(worker = "tiingo", "websocket connected");
                backoff.reset();
                stream
            }
            Err(e) => {
                let delay = backoff.next_delay();
                error!(
                    worker = "tiingo",
                    error = %e,
                    retry_secs = delay.as_secs(),
                    "websocket connection failed"
                );
                tokio::time::sleep(delay).await;
                continue;
            }
        };

        let (mut write, mut read) = ws_stream.split();

        let sub_msg = json!({
            "eventName": "subscribe",
            "authorization": &cfg.tiingo_api_key,
            "eventData": {
                "thresholdLevel": 5,
                "tickers": TICKERS
            }
        });

        if let Err(e) = write.send(Message::Text(sub_msg.to_string().into())).await {
            error!(worker = "tiingo", error = %e, "failed to send subscribe message");
            let delay = backoff.next_delay();
            tokio::time::sleep(delay).await;
            continue;
        }

        info!(
            worker = "tiingo",
            tickers = ?TICKERS,
            threshold_level = 5,
            "subscribed"
        );

        let check_interval_dur = Duration::from_secs(cfg.market_check_interval_sec);
        let mut market_check = interval(check_interval_dur);
        market_check.set_missed_tick_behavior(MissedTickBehavior::Skip);
        let mut tick_count: u64 = 0;

        let disconnect_reason: &str;

        loop {
            tokio::select! {
                msg = read.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            if let Err(e) = handle_message(&text, &*broker, &mut tick_count).await {
                                debug!(worker = "tiingo", error = %e, "message handling error");
                            }
                        }
                        Some(Ok(Message::Ping(data))) => {
                            if let Err(e) = write.send(Message::Pong(data)).await {
                                warn!(worker = "tiingo", error = %e, "pong send failed");
                            }
                        }
                        Some(Ok(Message::Close(_))) => {
                            info!(worker = "tiingo", "server sent close frame");
                            disconnect_reason = "server_close";
                            break;
                        }
                        Some(Err(e)) => {
                            error!(worker = "tiingo", error = %e, "websocket read error");
                            disconnect_reason = "read_error";
                            break;
                        }
                        None => {
                            info!(worker = "tiingo", "websocket stream ended");
                            disconnect_reason = "stream_end";
                            break;
                        }
                        _ => {}
                    }
                }
                _ = market_check.tick() => {
                    if !market_hours::is_market_open() {
                        info!(
                            worker = "tiingo",
                            ticks_received = tick_count,
                            "market closed, disconnecting"
                        );
                        disconnect_reason = "market_closed";
                        break;
                    }
                }
            }
        }

        info!(
            worker = "tiingo",
            reason = disconnect_reason,
            ticks = tick_count,
            "disconnecting websocket"
        );

        let _ = write.send(Message::Close(None)).await;
        let _ = write.close().await;

        if disconnect_reason == "market_closed" {
            continue;
        }

        let delay = backoff.next_delay();
        warn!(
            worker = "tiingo",
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
    let msg: Value = serde_json::from_str(text)?;

    let msg_type = msg
        .get("messageType")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    match msg_type {
        "I" => {
            info!(worker = "tiingo", "subscription confirmed");
            return Ok(());
        }
        "H" => {
            debug!(worker = "tiingo", "heartbeat received");
            return Ok(());
        }
        "A" => {
        }
        other => {
            debug!(worker = "tiingo", msg_type = other, "unknown message type");
            return Ok(());
        }
    }

    let data = match msg.get("data").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return Ok(()),
    };

    if data.len() < 7 {
        debug!(worker = "tiingo", len = data.len(), "data array too short");
        return Ok(());
    }

    let service_type = data[0].as_str().unwrap_or("");
    if service_type != "Q" {
        debug!(worker = "tiingo", service = service_type, "non-quote data");
        return Ok(());
    }

    let ticker = data[1].as_str().unwrap_or("unknown");
    let datetime = data[2].as_str().unwrap_or("");
    let bid_price = data[3].as_f64();
    let mid_price = data[5].as_f64();
    let ask_price = data[6].as_f64();

    let price = mid_price
        .or(bid_price)
        .or(ask_price)
        .unwrap_or(0.0);

    if price <= 0.0 {
        return Ok(());
    }

    *tick_count += 1;

    let payload = json!({
        "source": "tiingo",
        "symbol": ticker,
        "price": price,
        "bid": bid_price,
        "ask": ask_price,
        "mid": mid_price,
        "provider_timestamp": datetime,
        "received_at": Utc::now().to_rfc3339(),
    });

    let payload_str = payload.to_string();
    if let Err(e) = broker.publish(TOPIC, &payload_str).await {
        warn!(
            worker = "tiingo",
            error = %e,
            ticker = ticker,
            "broker publish failed"
        );
    }

    Ok(())
}
