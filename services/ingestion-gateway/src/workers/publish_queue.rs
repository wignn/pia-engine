use std::{sync::Arc, time::Duration};

use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::broker::BrokerPublisher;
use crate::health::HealthRegistry;

#[derive(Debug)]
pub struct PublishEvent {
    pub subject: &'static str,
    pub payload: String,
    pub symbol: String,
}

pub type PublishQueue = mpsc::Sender<PublishEvent>;

pub fn spawn_publisher(
    worker: &'static str,
    broker: Arc<dyn BrokerPublisher>,
    health: HealthRegistry,
    capacity: usize,
    publish_timeout: Duration,
    progress_log_interval: u64,
) -> PublishQueue {
    let (tx, mut rx) = mpsc::channel::<PublishEvent>(capacity);
    let queue_health = health.clone();

    tokio::spawn(async move {
        queue_health.set_queue_capacity(worker, capacity).await;
    });

    tokio::spawn(async move {
        let mut published_count = 0_u64;

        while let Some(event) = rx.recv().await {
            match tokio::time::timeout(
                publish_timeout,
                broker.publish(event.subject, &event.payload),
            )
            .await
            {
                Ok(Ok(())) => {
                    published_count += 1;
                    health.record_published(worker, rx.len()).await;
                    if published_count % progress_log_interval == 0 {
                        info!(
                            worker,
                            subject = event.subject,
                            published = published_count,
                            "market data events published"
                        );
                    }
                }
                Ok(Err(err)) => {
                    health.record_publish_failure(worker, rx.len()).await;
                    warn!(worker, error = %err, symbol = %event.symbol, "broker publish failed");
                }
                Err(_) => {
                    health.record_publish_timeout(worker, rx.len()).await;
                    warn!(
                        worker,
                        symbol = %event.symbol,
                        timeout_secs = publish_timeout.as_secs(),
                        "broker publish timed out"
                    );
                }
            }
        }

        error!(worker, "publish queue closed");
    });

    tx
}

pub fn enqueue_or_drop(
    worker: &'static str,
    queue: &PublishQueue,
    event: PublishEvent,
    queued_count: &mut u64,
) -> bool {
    match queue.try_send(event) {
        Ok(()) => {
            *queued_count += 1;
            true
        }
        Err(mpsc::error::TrySendError::Full(event)) => {
            warn!(
                worker,
                subject = event.subject,
                symbol = %event.symbol,
                queued = *queued_count,
                "publish queue full, dropping market data event"
            );
            false
        }
        Err(mpsc::error::TrySendError::Closed(event)) => {
            error!(
                worker,
                subject = event.subject,
                symbol = %event.symbol,
                "publish queue closed, dropping market data event"
            );
            false
        }
    }
}
