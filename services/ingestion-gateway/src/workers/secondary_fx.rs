use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use atlsd_eventbus::subjects;
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::time::{interval, MissedTickBehavior};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

use super::reconnect::ReconnectPolicy;
use crate::broker::BrokerPublisher;
use crate::config::{Config, MarketSymbolConfig};
use crate::market_hours;

const WORKER: &str = "secondary_fx";
const FEED: &str = "secondary_fx";
const SOURCE: &str = "market_data";
const TOPIC: &str = subjects::MD_RAW_SECONDARY_FX_QUOTES_V1;

pub async fn run(cfg: Arc<Config>, broker: Arc<dyn BrokerPublisher>) {
    let mut backoff = ReconnectPolicy::new(cfg.reconnect_base_sec, cfg.reconnect_max_sec);

    loop {
        if !market_hours::is_market_open() {
            let wait = market_hours::duration_until_next_open();
            info!(
                worker = WORKER,
                wait_secs = wait.as_secs(),
                "market closed, sleeping until open"
            );
            tokio::time::sleep(wait).await;
            continue;
        }

        if cfg.secondary_fx_ws_url.trim().is_empty() {
            error!(worker = WORKER, "secondary FX websocket URL not configured");
            tokio::time::sleep(backoff.next_delay()).await;
            continue;
        }

        info!(
            worker = WORKER,
            symbols = cfg.secondary_fx_symbols.len(),
            "connecting to market data websocket"
        );

        let ws_stream = match connect_async(cfg.secondary_fx_ws_url.trim()).await {
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
        let provider_symbols: Vec<&str> = cfg
            .secondary_fx_symbols
            .iter()
            .map(|symbol| symbol.provider_symbol.as_str())
            .collect();

        let sub_msg = json!({
            "eventName": "subscribe",
            "authorization": &cfg.secondary_fx_api_key,
            "eventData": {
                "thresholdLevel": 5,
                "tickers": provider_symbols
            }
        });

        if let Err(e) = write.send(Message::Text(sub_msg.to_string())).await {
            error!(worker = WORKER, error = %e, "failed to send subscribe message");
            let delay = backoff.next_delay();
            tokio::time::sleep(delay).await;
            continue;
        }

        info!(
            worker = WORKER,
            symbols = cfg.secondary_fx_symbols.len(),
            threshold_level = 5,
            "subscribed"
        );

        let symbol_map = symbol_map(&cfg.secondary_fx_symbols);
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
                            if let Err(e) = handle_message(&text, &symbol_map, &*broker, &mut tick_count).await {
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
                _ = market_check.tick() => {
                    if !market_hours::is_market_open() {
                        info!(worker = WORKER, ticks_received = tick_count, "market closed, disconnecting");
                        disconnect_reason = "market_closed";
                        break;
                    }
                }
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

        if disconnect_reason == "market_closed" {
            continue;
        }

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
    symbol_map: &HashMap<String, MarketSymbolConfig>,
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
            info!(worker = WORKER, "subscription confirmed");
            return Ok(());
        }
        "H" => {
            debug!(worker = WORKER, "heartbeat received");
            return Ok(());
        }
        "A" => {}
        other => {
            debug!(worker = WORKER, msg_type = other, "unknown message type");
            return Ok(());
        }
    }

    let data = match msg.get("data").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return Ok(()),
    };

    if data.len() < 7 {
        debug!(worker = WORKER, len = data.len(), "data array too short");
        return Ok(());
    }

    let service_type = data[0].as_str().unwrap_or("");
    if service_type != "Q" {
        debug!(worker = WORKER, service = service_type, "non-quote data");
        return Ok(());
    }

    let ticker = data[1].as_str().unwrap_or("unknown");
    let Some(symbol) = symbol_map.get(ticker) else {
        debug!(worker = WORKER, "unmapped symbol received");
        return Ok(());
    };

    let datetime = data[2].as_str().unwrap_or("");
    let bid_price = data[3].as_f64();
    let mid_price = data[5].as_f64();
    let ask_price = data[6].as_f64();

    let price = mid_price.or(bid_price).or(ask_price).unwrap_or(0.0);

    if price <= 0.0 {
        return Ok(());
    }

    *tick_count += 1;

    let payload = json!({
        "feed": FEED,
        "source": SOURCE,
        "symbol": symbol.public_symbol,
        "asset_type": symbol.asset_type,
        "price": price,
        "bid": bid_price,
        "ask": ask_price,
        "mid": mid_price,
        "provider_timestamp": datetime,
        "received_at": Utc::now().to_rfc3339(),
    });

    let payload_str = payload.to_string();
    if let Err(e) = broker.publish(TOPIC, &payload_str).await {
        warn!(worker = WORKER, error = %e, symbol = %symbol.public_symbol, "broker publish failed");
    }

    Ok(())
}

fn symbol_map(symbols: &[MarketSymbolConfig]) -> HashMap<String, MarketSymbolConfig> {
    symbols
        .iter()
        .map(|symbol| (symbol.provider_symbol.clone(), symbol.clone()))
        .collect()
}
