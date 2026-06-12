use atlsd_eventbus::{subjects, EventBusMode};
use futures_util::StreamExt;
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::{config::Config, hub::Hub};

const SUBJECTS: &[&str] = &[
    subjects::MD_DEDUP_PRIMARY_FX_QUOTES_V1,
    subjects::MD_DEDUP_CRYPTO_TRADES_V1,
    subjects::MD_DEDUP_INDEX_QUOTES_V1,
    subjects::MARKET_ALERTS_V1,
    subjects::NEWS_FOREX_PROCESSED_V1,
    subjects::NEWS_STOCK_PROCESSED_V1,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn market_subjects_use_deduplicated_streams() {
        assert!(SUBJECTS.contains(&subjects::MD_DEDUP_PRIMARY_FX_QUOTES_V1));
        assert!(!SUBJECTS.contains(&subjects::MD_RAW_PRIMARY_FX_QUOTES_V1));
    }
}

pub async fn run(cfg: Config, hub: Arc<Hub>) {
    match EventBusMode::from_env_value(&cfg.eventbus_mode) {
        EventBusMode::Nats | EventBusMode::Dual => run_loop(cfg, hub).await,
        EventBusMode::Redis | EventBusMode::Noop => {}
    }
}

async fn run_loop(cfg: Config, hub: Arc<Hub>) {
    loop {
        if let Err(err) = subscribe_loop(&cfg.nats_url, &hub).await {
            error!(error = %err, "realtime NATS subscriber failed, reconnecting in 5s");
        }
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}

async fn subscribe_loop(nats_url: &str, hub: &Arc<Hub>) -> anyhow::Result<()> {
    let client = async_nats::connect(nats_url).await?;
    atlsd_eventbus::nats::init_jetstream_streams(&client).await?;
    let mut subscribers = futures_util::stream::SelectAll::new();
    for subject in SUBJECTS {
        subscribers.push(client.subscribe((*subject).to_string()).await?);
    }
    info!(subjects = ?SUBJECTS, "realtime gateway subscribed to NATS subjects");

    while let Some(message) = subscribers.next().await {
        let subject = message.subject.as_str();
        let payload = std::str::from_utf8(&message.payload)?;
        match subject {
            subjects::NEWS_FOREX_PROCESSED_V1 => {
                broadcast_news(hub, "forex_news.new", payload, "forex_news").await
            }
            subjects::NEWS_STOCK_PROCESSED_V1 => {
                broadcast_news(hub, "stock.news.new", payload, "stock_news").await
            }
            subjects::MARKET_ALERTS_V1 => {
                broadcast_news(hub, "market.alert", payload, "market_alerts").await
            }
            _ => match market_tick(payload) {
                Ok(Some(tick)) => {
                    hub.broadcast("market.trade", json!({ "tick": tick }), "market_data")
                        .await;
                }
                Ok(None) => {}
                Err(err) => warn!(error = %err, "failed to parse NATS market payload"),
            },
        }
    }

    Ok(())
}

async fn broadcast_news(hub: &Arc<Hub>, event: &str, payload: &str, channel: &str) {
    match serde_json::from_str::<Value>(payload) {
        Ok(data) => {
            hub.broadcast(event, data, channel).await;
        }
        Err(err) => warn!(error = %err, channel, "failed to parse NATS news payload"),
    }
}

fn market_tick(payload: &str) -> anyhow::Result<Option<Value>> {
    let mut tick: Value = serde_json::from_str(payload)?;
    let Some(object) = tick.as_object_mut() else {
        return Ok(None);
    };

    let price = object
        .get("price")
        .and_then(|value| value.as_f64())
        .unwrap_or(0.0);
    if price <= 0.0 {
        return Ok(None);
    }

    object.insert("source".to_string(), json!("market_data"));
    Ok(Some(tick))
}
