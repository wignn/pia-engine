use atlsd_eventbus::nats::{durable_pull_consumer, init_jetstream_streams};
use atlsd_eventbus::{subjects, EventBusMode};
use futures_util::StreamExt;
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::{config::Config, hub::Hub};

const NEWS_CONSUMER: &str = "realtime-news";
const MARKET_CONSUMER: &str = "realtime-market";
const CANDLE_CONSUMER: &str = "realtime-candles";

const CORE_SUBJECTS: &[&str] = &[subjects::MARKET_ALERTS_V1, subjects::SOCIAL_POSTS];

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
    init_jetstream_streams(&client).await?;

    let news = durable_pull_consumer(
        &client,
        subjects::ATLSD_NEWS_STREAM,
        NEWS_CONSUMER,
        "news.>",
    )
    .await?;
    let mut news_messages = news.messages().await?;
    let market = durable_pull_consumer(
        &client,
        subjects::ATLSD_MARKET_DEDUP_STREAM,
        MARKET_CONSUMER,
        "md.dedup.>",
    )
    .await?;
    let mut market_messages = market.messages().await?;
    let candles = durable_pull_consumer(
        &client,
        subjects::ATLSD_MARKET_DEDUP_STREAM,
        CANDLE_CONSUMER,
        "md.candle.>",
    )
    .await?;
    let mut candle_messages = candles.messages().await?;

    let mut core = futures_util::stream::SelectAll::new();
    for subject in CORE_SUBJECTS {
        core.push(client.subscribe((*subject).to_string()).await?);
    }

    info!(
        news_consumer = NEWS_CONSUMER,
        market_consumer = MARKET_CONSUMER,
        candle_consumer = CANDLE_CONSUMER,
        core_subjects = ?CORE_SUBJECTS,
        "realtime gateway consuming durable JetStream consumers + core subjects"
    );

    loop {
        tokio::select! {
            message = news_messages.next() => match message {
                Some(Ok(message)) => handle_and_ack(hub, &message).await,
                Some(Err(err)) => warn!(error = %err, "news consumer stream error"),
                None => return Ok(()),
            },
            message = market_messages.next() => match message {
                Some(Ok(message)) => handle_and_ack(hub, &message).await,
                Some(Err(err)) => warn!(error = %err, "market consumer stream error"),
                None => return Ok(()),
            },
            message = candle_messages.next() => match message {
                Some(Ok(message)) => handle_and_ack(hub, &message).await,
                Some(Err(err)) => warn!(error = %err, "candle consumer stream error"),
                None => return Ok(()),
            },
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

async fn handle_and_ack(hub: &Arc<Hub>, message: &async_nats::jetstream::Message) {
    let subject = message.message.subject.as_str();
    match std::str::from_utf8(&message.message.payload) {
        Ok(payload) => dispatch(hub, subject, payload).await,
        Err(err) => warn!(error = %err, subject, "JetStream payload was not UTF-8"),
    }

    if let Err(err) = message.ack().await {
        warn!(error = %err, subject, "failed to ack JetStream message; it will be redelivered");
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
    fn core_subjects_exclude_stream_backed_subjects() {
        assert!(!CORE_SUBJECTS
            .iter()
            .any(|subject| subject.starts_with("md.")));
        assert!(!CORE_SUBJECTS
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
}
