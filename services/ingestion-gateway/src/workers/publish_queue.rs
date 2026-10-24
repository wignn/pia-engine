use std::{sync::Arc, time::Duration};

use atlsd_eventbus::EventPublisher;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::health::HealthRegistry;

#[derive(Debug)]
pub struct PublishEvent {
    pub subject: &'static str,
    pub payload: String,
    pub symbol: String,
    pub msg_id: Option<String>,
}

pub type PublishQueue = mpsc::Sender<PublishEvent>;

pub fn market_data_msg_id(symbol: &str, timestamp_ms: i64, price: f64, volume: f64) -> String {
    format!("{symbol}:{timestamp_ms}:{price}:{volume}")
}

pub fn spawn_publisher(
    worker: &'static str,
    broker: Arc<dyn EventPublisher>,
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
            let publish = async {
                match event.msg_id.as_deref() {
                    Some(msg_id) => {
                        broker
                            .publish_str_with_id(event.subject, &event.payload, msg_id)
                            .await
                    }
                    None => broker.publish_str(event.subject, &event.payload).await,
                }
            };

            match tokio::time::timeout(publish_timeout, publish).await {
                Ok(Ok(())) => {
                    published_count += 1;
                    health.record_published(worker, rx.len()).await;
                    if published_count.is_multiple_of(progress_log_interval) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn market_data_msg_id_is_deterministic() {
        assert_eq!(
            market_data_msg_id("XAUUSD", 1_710_000_000_000, 4204.795, 1.0),
            "XAUUSD:1710000000000:4204.795:1"
        );
    }
}
