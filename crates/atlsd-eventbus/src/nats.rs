use async_trait::async_trait;

use crate::publisher::EventPublisher;

#[derive(Clone)]
pub struct NatsPublisher {
    client: async_nats::Client,
}

impl NatsPublisher {
    pub async fn connect(url: &str) -> anyhow::Result<Self> {
        Ok(Self {
            client: async_nats::connect(url).await?,
        })
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
}
