use async_nats::jetstream::consumer::{self, DeliverPolicy};
use async_nats::jetstream::stream::Config as StreamConfig;
use async_nats::jetstream::Context as JetStreamContext;
use async_trait::async_trait;
use std::time::Duration;

use crate::publisher::EventPublisher;
use crate::subjects;

const NATS_MSG_ID: &str = "Nats-Msg-Id";
const MARKET_DEDUP_WINDOW: Duration = Duration::from_secs(120);
const MARKET_MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);
const MARKET_MAX_BYTES: i64 = 2 * 1024 * 1024 * 1024;
const MARKET_DEDUP_MAX_AGE: Duration = Duration::from_secs(12 * 60 * 60);
const MARKET_DEDUP_MAX_BYTES: i64 = 1024 * 1024 * 1024;
const NEWS_MAX_AGE: Duration = Duration::from_secs(7 * 24 * 60 * 60);
const INTELLIGENCE_MAX_AGE: Duration = Duration::from_secs(7 * 24 * 60 * 60);
const PLATFORM_MAX_AGE: Duration = Duration::from_secs(30 * 24 * 60 * 60);

#[derive(Clone)]
pub struct NatsPublisher {
    context: JetStreamContext,
}

pub fn market_stream_config() -> StreamConfig {
    StreamConfig {
        name: subjects::ATLSD_MARKET_STREAM.to_string(),
        subjects: vec!["md.raw.>".to_string()],
        duplicate_window: MARKET_DEDUP_WINDOW,
        max_age: MARKET_MAX_AGE,
        max_bytes: MARKET_MAX_BYTES,
        ..Default::default()
    }
}

pub fn market_dedup_stream_config() -> StreamConfig {
    StreamConfig {
        name: subjects::ATLSD_MARKET_DEDUP_STREAM.to_string(),
        subjects: vec!["md.dedup.>".to_string(), "md.candle.>".to_string()],
        max_age: MARKET_DEDUP_MAX_AGE,
        max_bytes: MARKET_DEDUP_MAX_BYTES,
        ..Default::default()
    }
}

pub fn news_stream_config() -> StreamConfig {
    StreamConfig {
        name: subjects::ATLSD_NEWS_STREAM.to_string(),
        subjects: vec!["news.>".to_string()],
        max_age: NEWS_MAX_AGE,
        ..Default::default()
    }
}

pub fn intelligence_stream_config() -> StreamConfig {
    StreamConfig {
        name: subjects::ATLSD_INTELLIGENCE_STREAM.to_string(),
        subjects: vec!["intel.>".to_string()],
        max_age: INTELLIGENCE_MAX_AGE,
        ..Default::default()
    }
}

pub fn platform_stream_config() -> StreamConfig {
    StreamConfig {
        name: subjects::ATLSD_PLATFORM_STREAM.to_string(),
        subjects: vec![
            "tenant.>".to_string(),
            "usage.>".to_string(),
            "audit.>".to_string(),
            "platform.>".to_string(),
        ],
        max_age: PLATFORM_MAX_AGE,
        ..Default::default()
    }
}

pub fn dedup_headers(msg_id: &str) -> async_nats::HeaderMap {
    let mut headers = async_nats::HeaderMap::new();
    headers.insert(NATS_MSG_ID, msg_id);
    headers
}

pub async fn init_jetstream_streams(client: &async_nats::Client) -> anyhow::Result<()> {
    let context = async_nats::jetstream::new(client.clone());
    let configs = [
        market_stream_config(),
        market_dedup_stream_config(),
        news_stream_config(),
        intelligence_stream_config(),
        platform_stream_config(),
    ];

    for config in configs {
        let mut stream = context.get_or_create_stream(config.clone()).await?;
        let info = stream.info().await?;
        tracing::info!(
            stream = %info.config.name,
            messages = info.state.messages,
            "JetStream stream ready"
        );
    }

    Ok(())
}

pub async fn durable_pull_consumer(
    client: &async_nats::Client,
    stream: &str,
    durable: &str,
    filter_subject: &str,
) -> anyhow::Result<consumer::Consumer<consumer::pull::Config>> {
    let context = async_nats::jetstream::new(client.clone());
    let stream = context.get_stream(stream.to_string()).await?;
    let config = consumer::pull::Config {
        durable_name: Some(durable.to_string()),
        filter_subject: filter_subject.to_string(),
        deliver_policy: DeliverPolicy::New,
        ack_wait: Duration::from_secs(30),
        ..Default::default()
    };

    Ok(stream.get_or_create_consumer(durable, config).await?)
}

impl NatsPublisher {
    pub async fn connect(url: &str) -> anyhow::Result<Self> {
        let client = async_nats::connect(url).await?;
        init_jetstream_streams(&client).await?;
        Ok(Self {
            context: async_nats::jetstream::new(client),
        })
    }

    pub fn from_client(client: async_nats::Client) -> Self {
        Self {
            context: async_nats::jetstream::new(client),
        }
    }

    pub fn context(&self) -> &JetStreamContext {
        &self.context
    }
}

#[async_trait]
impl EventPublisher for NatsPublisher {
    async fn publish_str(&self, subject: &str, payload: &str) -> anyhow::Result<()> {
        let ack = self
            .context
            .publish(subject.to_string(), payload.as_bytes().to_vec().into())
            .await?
            .await?;
        tracing::debug!(
            subject,
            bytes = payload.len(),
            stream = %ack.stream,
            "published to JetStream"
        );
        Ok(())
    }

    async fn publish_str_with_id(
        &self,
        subject: &str,
        payload: &str,
        msg_id: &str,
    ) -> anyhow::Result<()> {
        let ack = self
            .context
            .publish_with_headers(
                subject.to_string(),
                dedup_headers(msg_id),
                payload.as_bytes().to_vec().into(),
            )
            .await?
            .await?;
        tracing::debug!(
            subject,
            msg_id,
            bytes = payload.len(),
            stream = %ack.stream,
            duplicate = ack.duplicate,
            "published to JetStream with message ID"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn market_stream_config_captures_raw_market_subjects() {
        let config = market_stream_config();

        assert_eq!(config.name, subjects::ATLSD_MARKET_STREAM);
        assert!(config.subjects.contains(&"md.raw.>".to_string()));
        assert!(!config.duplicate_window.is_zero());
        assert!(config.republish.is_none());
    }

    #[test]
    fn market_dedup_stream_config_captures_dedup_subjects() {
        let config = market_dedup_stream_config();

        assert_eq!(config.name, subjects::ATLSD_MARKET_DEDUP_STREAM);
        assert!(config.subjects.contains(&"md.dedup.>".to_string()));
        assert!(config.subjects.contains(&"md.candle.>".to_string()));
        assert!(config.max_age > Duration::ZERO);
        assert!(config.max_bytes > 0);
    }

    #[test]
    fn market_stream_config_bounds_retention() {
        let config = market_stream_config();

        assert!(config.max_age > Duration::ZERO);
        assert!(config.max_bytes > 0);
    }

    #[test]
    fn news_stream_config_captures_news_subjects() {
        let config = news_stream_config();

        assert_eq!(config.name, subjects::ATLSD_NEWS_STREAM);
        assert!(config.subjects.contains(&"news.>".to_string()));
        assert!(config.max_age > Duration::ZERO);
        assert!(config.republish.is_none());
    }

    #[test]
    fn intelligence_stream_config_captures_intel_subjects() {
        let config = intelligence_stream_config();

        assert_eq!(config.name, subjects::ATLSD_INTELLIGENCE_STREAM);
        assert_eq!(config.subjects, vec!["intel.>".to_string()]);
    }

    #[test]
    fn platform_stream_config_captures_platform_subjects() {
        let config = platform_stream_config();

        assert_eq!(config.name, subjects::ATLSD_PLATFORM_STREAM);
        assert!(config.subjects.contains(&"tenant.>".to_string()));
        assert!(config.subjects.contains(&"usage.>".to_string()));
        assert!(config.subjects.contains(&"platform.>".to_string()));
    }

    #[test]
    fn all_stream_configs_use_unique_names() {
        let names = [
            market_stream_config().name,
            market_dedup_stream_config().name,
            news_stream_config().name,
            intelligence_stream_config().name,
            platform_stream_config().name,
        ];

        let mut sorted: Vec<String> = names.to_vec();
        sorted.sort();
        sorted.dedup();
        assert_eq!(names.len(), sorted.len(), "stream names must be unique");
    }

    #[test]
    fn dedup_headers_sets_nats_message_id() {
        let headers = dedup_headers("XAUUSD:1710000000000:4204.795:1");

        assert_eq!(
            headers.get("Nats-Msg-Id").map(|value| value.as_str()),
            Some("XAUUSD:1710000000000:4204.795:1")
        );
    }
}
