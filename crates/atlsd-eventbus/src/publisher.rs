use async_trait::async_trait;

#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish_json(&self, subject: &str, payload: &serde_json::Value) -> anyhow::Result<()> {
        self.publish_str(subject, &serde_json::to_string(payload)?)
            .await
    }

    async fn publish_str(&self, subject: &str, payload: &str) -> anyhow::Result<()>;
}

pub struct NoopPublisher;

#[async_trait]
impl EventPublisher for NoopPublisher {
    async fn publish_str(&self, subject: &str, payload: &str) -> anyhow::Result<()> {
        tracing::debug!(subject, bytes = payload.len(), "noop eventbus publish");
        Ok(())
    }
}

#[async_trait]
impl EventPublisher for std::sync::Arc<dyn EventPublisher> {
    async fn publish_str(&self, subject: &str, payload: &str) -> anyhow::Result<()> {
        self.as_ref().publish_str(subject, payload).await
    }
}
pub struct DualPublisher<L, R> {
    left: L,
    right: R,
}

impl<L, R> DualPublisher<L, R> {
    pub fn new(left: L, right: R) -> Self {
        Self { left, right }
    }
}

#[async_trait]
impl<L, R> EventPublisher for DualPublisher<L, R>
where
    L: EventPublisher,
    R: EventPublisher,
{
    async fn publish_str(&self, subject: &str, payload: &str) -> anyhow::Result<()> {
        let left = self.left.publish_str(subject, payload).await;
        let right = self.right.publish_str(subject, payload).await;

        match (left, right) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(left), Ok(())) => Err(left),
            (Ok(()), Err(right)) => Err(right),
            (Err(left), Err(right)) => Err(anyhow::anyhow!("dual publish failed: {left}; {right}")),
        }
    }
}
