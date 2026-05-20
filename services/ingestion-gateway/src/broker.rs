use anyhow::Result;
use async_trait::async_trait;
use tracing::{debug, warn};

#[async_trait]
pub trait BrokerPublisher: Send + Sync + 'static {
    async fn publish(&self, topic: &str, payload: &str) -> Result<()>;
}

pub struct RedisBrokerPublisher {
    client: redis::Client,
    prefix: String,
}

impl RedisBrokerPublisher {
    pub fn new(client: redis::Client, prefix: String) -> Self {
        Self { client, prefix }
    }

    fn full_channel(&self, topic: &str) -> String {
        if self.prefix.is_empty() {
            topic.to_string()
        } else {
            format!("{}:{}", self.prefix, topic)
        }
    }
}

#[async_trait]
impl BrokerPublisher for RedisBrokerPublisher {
    async fn publish(&self, topic: &str, payload: &str) -> Result<()> {
        let channel = self.full_channel(topic);

        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| {
                warn!(error = %e, channel = %channel, "redis connection failed");
                e
            })?;

        let _: i64 = redis::cmd("PUBLISH")
            .arg(&channel)
            .arg(payload)
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                warn!(error = %e, channel = %channel, "redis PUBLISH failed");
                e
            })?;

        debug!(channel = %channel, bytes = payload.len(), "published to redis");
        Ok(())
    }
}

pub struct NoopBrokerPublisher;

#[async_trait]
impl BrokerPublisher for NoopBrokerPublisher {
    async fn publish(&self, topic: &str, payload: &str) -> Result<()> {
        debug!(
            topic = %topic,
            payload_len = payload.len(),
            "noop broker publish (no broker configured)"
        );
        Ok(())
    }
}
