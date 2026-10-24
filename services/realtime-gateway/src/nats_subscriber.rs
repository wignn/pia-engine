use atlsd_eventbus::{subjects, EventBusMode};
use futures_util::StreamExt;
use serde_json::{json, Value};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tracing::{error, info, warn};

use crate::{config::Config, hub::Hub};

const FANOUT_SUBJECTS: &[&str] = &[
    "news.>",
    "md.dedup.>",
    "md.candle.>",
    subjects::MARKET_ALERTS_V1,
    subjects::SOCIAL_POSTS,
];

pub async fn run(cfg: Config, hub: Arc<Hub>, ready: Arc<AtomicBool>) {
    match EventBusMode::from_env_value(&cfg.eventbus_mode) {
        EventBusMode::Nats | EventBusMode::Dual => run_loop(cfg, hub, ready).await,
        EventBusMode::Redis | EventBusMode::Noop => {}
    }
}

async fn run_loop(cfg: Config, hub: Arc<Hub>, ready: Arc<AtomicBool>) {
    loop {
        if let Err(err) = subscribe_loop(&cfg.nats_url, &hub, &ready).await {
            error!(error = %err, "realtime NATS subscriber failed, reconnecting in 5s");
        }
        ready.store(false, Ordering::Release);
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}

async fn subscribe_loop(nats_url: &str, hub: &Arc<Hub>, ready: &AtomicBool) -> anyhow::Result<()> {
    let client = async_nats::connect(nats_url).await?;
    let mut core = futures_util::stream::SelectAll::new();
    for subject in FANOUT_SUBJECTS {
        core.push(client.subscribe((*subject).to_string()).await?);
    }
    ready.store(true, Ordering::Release);

    info!(
        subjects = ?FANOUT_SUBJECTS,
        "realtime gateway subscribed for per-instance NATS fanout"
    );

    loop {
        tokio::select! {
            message = core.next() => match message {
                Some(message) => {
                    let subject = message.subject.as_str();
                    let payload = std::str::from_utf8(&message.payload)?;
                    dispatch(hub, subject, payload).await;
                }
                None => return Ok(()),
            },
        }
    }
}

async fn dispatch(hub: &Arc<Hub>, subject: &str, payload: &str) {
    match subject {
        subjects::NEWS_FOREX_PROCESSED_V1 => {
            broadcast_news(hub, "forex_news.new", payload, "forex_news").await
        }
        subjects::NEWS_STOCK_PROCESSED_V1 => {
            broadcast_news(hub, "stock.news.new", payload, "stock_news").await
        }
        subjects::SOCIAL_POSTS => broadcast_social(hub, payload).await,
        subjects::MARKET_ALERTS_V1 => {
            broadcast_news(hub, "market.alert", payload, "market_alerts").await
        }
        subjects::MD_CANDLE_1M_V1 => {
            // Closed and corrected candles share one subject; consumers apply
            // the highest sequence per (symbol, bucket_start).
            broadcast_news(hub, "market.candle", payload, "market_data").await
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

async fn broadcast_news(hub: &Arc<Hub>, event: &str, payload: &str, channel: &str) {
    match serde_json::from_str::<Value>(payload) {
        Ok(data) => {
            hub.broadcast(event, data, channel).await;
        }
        Err(err) => warn!(error = %err, channel, "failed to parse NATS news payload"),
    }
}

async fn broadcast_social(hub: &Arc<Hub>, payload: &str) {
    let post = match serde_json::from_str::<Value>(payload) {
        Ok(Value::Object(post)) => Value::Object(post),
        Ok(_) => {
            warn!("failed to parse NATS social payload: expected JSON object");
            return;
        }
        Err(err) => {
            warn!(error = %err, "failed to parse NATS social payload");
            return;
        }
    };

    let (event, channel) = social_route(&post);
    hub.broadcast(event, serde_json::json!({ "post": post }), channel)
        .await;
}

fn social_route(post: &Value) -> (&'static str, &'static str) {
    if post.get("platform").and_then(Value::as_str) == Some("twitter") {
        ("x.post", "x")
    } else {
        ("social.post", "social")
    }
}

fn market_tick(payload: &str) -> anyhow::Result<Option<Value>> {
    let mut tick: Value = serde_json::from_str(payload)?;
    if let Some(envelope_payload) = tick.get("payload").cloned() {
        tick = envelope_payload;
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subscribes_to_live_fanout_subjects_not_raw_ingestion() {
        assert!(!FANOUT_SUBJECTS
            .iter()
            .any(|subject| subject.starts_with("md.raw.")));
        assert!(FANOUT_SUBJECTS
            .iter()
            .any(|subject| subject.starts_with("md.dedup.")));
        assert!(FANOUT_SUBJECTS
            .iter()
            .any(|subject| subject.starts_with("news.")));
    }

    #[test]
    fn social_posts_route_twitter_to_existing_x_channel() {
        let post = serde_json::json!({"platform": "twitter"});
        assert_eq!(social_route(&post), ("x.post", "x"));
    }

    #[test]
    fn social_posts_route_other_platforms_to_social_channel() {
        let post = serde_json::json!({"platform": "truth"});
        assert_eq!(social_route(&post), ("social.post", "social"));
    }

    #[test]
    fn market_tick_accepts_the_versioned_raw_event_contract() {
        let payload = serde_json::json!({
            "event_id": "00000000-0000-0000-0000-000000000001",
            "event_type": "md.raw.crypto.trades.v1",
            "schema_version": 1,
            "payload": {"symbol": "BTCUSDT", "price": 67000.0}
        })
        .to_string();

        let tick = market_tick(&payload).unwrap().unwrap();
        assert_eq!(tick["symbol"], "BTCUSDT");
        assert_eq!(tick["source"], "market_data");
    }
}
