use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use tokio::time::{interval, MissedTickBehavior};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

use super::reconnect::ReconnectPolicy;
use crate::broker::BrokerPublisher;
use crate::config::{Config, MarketSymbolConfig};
use crate::market_hours;

#[derive(Debug, Deserialize)]
struct ProviderTrade {
    p: f64,
    s: String,
    t: i64,
    v: f64,
}

#[derive(Debug, Deserialize)]
struct ProviderMessage {
    #[serde(rename = "type")]
    msg_type: String,
    #[serde(default)]
    data: Vec<ProviderTrade>,
}

const WORKER: &str = "primary_fx";
const FEED: &str = "primary_fx";
const SOURCE: &str = "market_data";
const TOPIC: &str = "primary_fx:prices";

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

        if cfg.primary_fx_ws_url.trim().is_empty() {
            error!(worker = WORKER, "primary FX websocket URL not configured");
            tokio::time::sleep(backoff.next_delay()).await;
            continue;
        }
        let url = cfg
            .primary_fx_ws_url
            .trim()
            .replace("{token}", cfg.primary_fx_api_key.trim());

        info!(
            worker = WORKER,
            symbols = cfg.primary_fx_symbols.len(),
            "connecting to market data websocket"
        );

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

        for symbol in &cfg.primary_fx_symbols {
            let sub_msg = json!({
                "type": "subscribe",
                "symbol": symbol.provider_symbol
            });

            if let Err(e) = write.send(Message::Text(sub_msg.to_string())).await {
                error!(worker = WORKER, error = %e, "failed to send subscribe message");
                let delay = backoff.next_delay();
                tokio::time::sleep(delay).await;
                continue;
            }
        }

        info!(
            worker = WORKER,
            symbols = cfg.primary_fx_symbols.len(),
            "subscribed"
        );

        let symbol_map = symbol_map(&cfg.primary_fx_symbols);
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

        for symbol in &cfg.primary_fx_symbols {
            let unsub_msg = json!({
                "type": "unsubscribe",
                "symbol": symbol.provider_symbol
            });
            let _ = write.send(Message::Text(unsub_msg.to_string())).await;
        }
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
    let msg: ProviderMessage = serde_json::from_str(text)?;

    if msg.msg_type == "ping" {
        return Ok(());
    }

    if msg.msg_type != "trade" {
        debug!(worker = WORKER, msg_type = %msg.msg_type, "non-trade message");
        return Ok(());
    }

    for trade in &msg.data {
        let Some(symbol) = symbol_map.get(&trade.s) else {
            debug!(worker = WORKER, "unmapped symbol received");
            continue;
        };

        *tick_count += 1;

        let payload = json!({
            "feed": FEED,
            "source": SOURCE,
            "symbol": symbol.public_symbol,
            "asset_type": symbol.asset_type,
            "price": trade.p,
            "volume": trade.v,
            "timestamp_ms": trade.t,
            "received_at": Utc::now().to_rfc3339(),
        });

        let payload_str = payload.to_string();
        if let Err(e) = broker.publish(TOPIC, &payload_str).await {
            warn!(worker = WORKER, error = %e, symbol = %symbol.public_symbol, "broker publish failed");
        }
    }

    Ok(())
}

fn symbol_map(symbols: &[MarketSymbolConfig]) -> HashMap<String, MarketSymbolConfig> {
    symbols
        .iter()
        .map(|symbol| (symbol.provider_symbol.clone(), symbol.clone()))
        .collect()
}
