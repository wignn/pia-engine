use async_trait::async_trait;

use crate::publisher::EventPublisher;

#[derive(Clone)]
pub struct RedisPublisher {
    client: redis::Client,
    prefix: String,
}

impl RedisPublisher {
    pub fn new(client: redis::Client, prefix: impl Into<String>) -> Self {
        Self {
            client,
            prefix: prefix.into(),
        }
    }

    fn full_channel(&self, subject: &str) -> String {
        if self.prefix.is_empty() {
            subject.to_string()
        } else {
            format!("{}:{}", self.prefix, subject)
        }
    }
}

#[async_trait]
impl EventPublisher for RedisPublisher {
    async fn publish_str(&self, subject: &str, payload: &str) -> anyhow::Result<()> {
        let channel = self.full_channel(subject);
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let _: i64 = redis::cmd("PUBLISH")
            .arg(&channel)
            .arg(payload)
            .query_async(&mut conn)
            .await?;
        tracing::debug!(channel = %channel, bytes = payload.len(), "published to Redis eventbus");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_channel_applies_prefix() {
        let publisher =
            RedisPublisher::new(redis::Client::open("redis://127.0.0.1/").unwrap(), "atlsd");
        assert_eq!(publisher.full_channel("md.raw"), "atlsd:md.raw");
    }
}
