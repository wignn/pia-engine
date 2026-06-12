use async_trait::async_trait;

#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish_json(&self, subject: &str, payload: &serde_json::Value) -> anyhow::Result<()> {
        self.publish_str(subject, &serde_json::to_string(payload)?)
            .await
    }

    async fn publish_str(&self, subject: &str, payload: &str) -> anyhow::Result<()>;

    async fn publish_str_with_id(
        &self,
        subject: &str,
        payload: &str,
        msg_id: &str,
    ) -> anyhow::Result<()> {
        let _ = msg_id;
        self.publish_str(subject, payload).await
    }
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
impl<T> EventPublisher for std::sync::Arc<T>
where
    T: EventPublisher + ?Sized,
{
    async fn publish_str(&self, subject: &str, payload: &str) -> anyhow::Result<()> {
        self.as_ref().publish_str(subject, payload).await
    }

    async fn publish_str_with_id(
        &self,
        subject: &str,
        payload: &str,
        msg_id: &str,
    ) -> anyhow::Result<()> {
        self.as_ref()
            .publish_str_with_id(subject, payload, msg_id)
            .await
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

        merge_dual_results(left, right)
    }

    async fn publish_str_with_id(
        &self,
        subject: &str,
        payload: &str,
        msg_id: &str,
    ) -> anyhow::Result<()> {
        let left = self
            .left
            .publish_str_with_id(subject, payload, msg_id)
            .await;
        let right = self
            .right
            .publish_str_with_id(subject, payload, msg_id)
            .await;

        merge_dual_results(left, right)
    }
}

fn merge_dual_results(left: anyhow::Result<()>, right: anyhow::Result<()>) -> anyhow::Result<()> {
    match (left, right) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(left), Ok(())) => Err(left),
        (Ok(()), Err(right)) => Err(right),
        (Err(left), Err(right)) => Err(anyhow::anyhow!("dual publish failed: {left}; {right}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct RecordingPublisher {
        calls: Mutex<Vec<(String, String, Option<String>)>>,
    }

    #[async_trait]
    impl EventPublisher for RecordingPublisher {
        async fn publish_str(&self, subject: &str, payload: &str) -> anyhow::Result<()> {
            self.calls
                .lock()
                .unwrap()
                .push((subject.to_string(), payload.to_string(), None));
            Ok(())
        }

        async fn publish_str_with_id(
            &self,
            subject: &str,
            payload: &str,
            msg_id: &str,
        ) -> anyhow::Result<()> {
            self.calls.lock().unwrap().push((
                subject.to_string(),
                payload.to_string(),
                Some(msg_id.to_string()),
            ));
            Ok(())
        }
    }

    #[tokio::test]
    async fn arc_publisher_forwards_message_ids() {
        let publisher = Arc::new(RecordingPublisher::default());
        let erased: Arc<dyn EventPublisher> = publisher.clone();

        erased
            .publish_str_with_id("subject", "payload", "dedup-id")
            .await
            .unwrap();

        assert_eq!(
            publisher.calls.lock().unwrap().as_slice(),
            &[(
                "subject".to_string(),
                "payload".to_string(),
                Some("dedup-id".to_string())
            )]
        );
    }

    #[tokio::test]
    async fn dual_publisher_forwards_message_ids_to_both_publishers() {
        let left = Arc::new(RecordingPublisher::default());
        let right = Arc::new(RecordingPublisher::default());
        let publisher = DualPublisher::new(left.clone(), right.clone());

        publisher
            .publish_str_with_id("subject", "payload", "dedup-id")
            .await
            .unwrap();

        assert_eq!(left.calls.lock().unwrap().len(), 1);
        assert_eq!(right.calls.lock().unwrap().len(), 1);
        assert_eq!(
            left.calls.lock().unwrap()[0].2,
            Some("dedup-id".to_string())
        );
        assert_eq!(
            right.calls.lock().unwrap()[0].2,
            Some("dedup-id".to_string())
        );
    }
}
