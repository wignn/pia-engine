use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use tokio::time::{interval, MissedTickBehavior};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

use crate::broker::BrokerPublisher;
use crate::config::Config;
use crate::market_hours;
use super::reconnect::ReconnectPolicy;

#[derive(Debug, Deserialize)]
struct FinnhubTrade {
    p: f64,
    s: String,
    t: i64,
    v: f64,
}
#[derive(Debug, Deserialize)]
struct FinnhubMessage {
    #[serde(rename = "type")]
    msg_type: String,
    #[serde(default)]
    data: Vec<FinnhubTrade>,
}

const SYMBOL: &str = "OANDA:XAU_USD";
const TOPIC: &str = "finnhub:xauusd";

pub async fn run(cfg: Arc<Config>, broker: Arc<dyn BrokerPublisher>) {
    let mut backoff = ReconnectPolicy::new(cfg.reconnect_base_sec, cfg.reconnect_max_sec);

    loop {
        if !market_hours::is_market_open() {
            let wait = market_hours::duration_until_next_open();
            info!(
                worker = "finnhub",
                wait_secs = wait.as_secs(),
                "forex market closed, sleeping until open"
            );
            tokio::time::sleep(wait).await;
            continue;
        }

        let url = format!("wss://ws.finnhub.io?token={}", cfg.finnhub_api_key);

        info!(worker = "finnhub", symbol = SYMBOL, "connecting to finnhub websocket");

        let ws_stream = match connect_async(&url).await {
            Ok((stream, _response)) => {
                info!(worker = "finnhub", "websocket connected");
                backoff.reset();
                stream
            }
            Err(e) => {
                let delay = backoff.next_delay();
                error!(
                    worker = "finnhub",
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
            "type": "subscribe",
            "symbol": SYMBOL
        });

        if let Err(e) = write.send(Message::Text(sub_msg.to_string().into())).await {
            error!(worker = "finnhub", error = %e, "failed to send subscribe message");
            let delay = backoff.next_delay();
            tokio::time::sleep(delay).await;
            continue;
        }

        info!(worker = "finnhub", symbol = SYMBOL, "subscribed");

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
                                debug!(worker = "finnhub", error = %e, "message handling error");
                            }
                        }
                        Some(Ok(Message::Ping(data))) => {
                            if let Err(e) = write.send(Message::Pong(data)).await {
                                warn!(worker = "finnhub", error = %e, "pong send failed");
                            }
                        }
                        Some(Ok(Message::Close(_))) => {
                            info!(worker = "finnhub", "server sent close frame");
                            disconnect_reason = "server_close";
                            break;
                        }
                        Some(Err(e)) => {
                            error!(worker = "finnhub", error = %e, "websocket read error");
                            disconnect_reason = "read_error";
                            break;
                        }
                        None => {
                            info!(worker = "finnhub", "websocket stream ended");
                            disconnect_reason = "stream_end";
                            break;
                        }
                        _ => {} 
                    }
                }
                _ = market_check.tick() => {
                    if !market_hours::is_market_open() {
                        info!(
                            worker = "finnhub",
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
            worker = "finnhub",
            reason = disconnect_reason,
            ticks = tick_count,
            "disconnecting websocket"
        );

        let unsub_msg = json!({
            "type": "unsubscribe",
            "symbol": SYMBOL
        });
        let _ = write.send(Message::Text(unsub_msg.to_string().into())).await;
        let _ = write.send(Message::Close(None)).await;
        let _ = write.close().await;

        if disconnect_reason == "market_closed" {
            continue;
        }

        let delay = backoff.next_delay();
        warn!(
            worker = "finnhub",
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
    let msg: FinnhubMessage = serde_json::from_str(text)?;

    if msg.msg_type == "ping" {
        return Ok(());
    }

    if msg.msg_type != "trade" {
        debug!(worker = "finnhub", msg_type = %msg.msg_type, "non-trade message");
        return Ok(());
    }

    for trade in &msg.data {
        *tick_count += 1;

        let payload = json!({
            "source": "finnhub",
            "symbol": &trade.s,
            "price": trade.p,
            "volume": trade.v,
            "timestamp_ms": trade.t,
            "received_at": Utc::now().to_rfc3339(),
        });

        let payload_str = payload.to_string();
        if let Err(e) = broker.publish(TOPIC, &payload_str).await {
            warn!(
                worker = "finnhub",
                error = %e,
                symbol = %trade.s,
                "broker publish failed"
            );
        }
    }

    Ok(())
}
