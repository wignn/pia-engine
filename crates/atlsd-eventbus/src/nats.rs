use async_nats::jetstream::stream::{Config as StreamConfig, Republish};
use async_trait::async_trait;
use std::time::Duration;

use crate::publisher::EventPublisher;

const NATS_MSG_ID: &str = "Nats-Msg-Id";
const MARKET_DEDUP_WINDOW: Duration = Duration::from_secs(120);

#[derive(Clone)]
pub struct NatsPublisher {
    client: async_nats::Client,
}

pub fn market_stream_config() -> StreamConfig {
    StreamConfig {
        name: crate::subjects::ATLSD_MARKET_STREAM.to_string(),
        subjects: vec!["md.raw.>".to_string()],
        duplicate_window: MARKET_DEDUP_WINDOW,
        republish: Some(Republish {
            source: "md.raw.>".to_string(),
            destination: "md.dedup.>".to_string(),
            headers_only: false,
        }),
        ..Default::default()
    }
}

pub fn dedup_headers(msg_id: &str) -> async_nats::HeaderMap {
    let mut headers = async_nats::HeaderMap::new();
    headers.insert(NATS_MSG_ID, msg_id);
    headers
}

pub async fn init_jetstream_streams(_client: &async_nats::Client) -> anyhow::Result<()> {
    Ok(())
}

impl NatsPublisher {
    pub async fn connect(url: &str) -> anyhow::Result<Self> {
        let client = async_nats::connect(url).await?;
        init_jetstream_streams(&client).await?;
        Ok(Self { client })
    }

    pub fn from_client(client: async_nats::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl EventPublisher for NatsPublisher {
    async fn publish_str(&self, subject: &str, payload: &str) -> anyhow::Result<()> {
        self.client
            .publish(subject.to_string(), payload.as_bytes().to_vec().into())
            .await?;
        tracing::debug!(subject, bytes = payload.len(), "published to NATS eventbus");
        Ok(())
    }

    async fn publish_str_with_id(
        &self,
        subject: &str,
        payload: &str,
        msg_id: &str,
    ) -> anyhow::Result<()> {
        self.client
            .publish_with_headers(
                subject.to_string(),
                dedup_headers(msg_id),
                payload.as_bytes().to_vec().into(),
            )
            .await?;
        tracing::debug!(
            subject,
            msg_id,
            bytes = payload.len(),
            "published to NATS eventbus with message ID"
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

        assert_eq!(config.name, crate::subjects::ATLSD_MARKET_STREAM);
        assert!(config.subjects.contains(&"md.raw.>".to_string()));
        assert!(!config.duplicate_window.is_zero());
        assert!(config.republish.is_some());
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
