use std::time::Duration;

use tokio::sync::mpsc;
use tracing::error;

pub struct BatcherConfig {
    pub max_batch_size: usize,
    pub max_delay: Duration,
}

pub async fn run_batcher<T, F, Fut>(mut rx: mpsc::Receiver<T>, config: BatcherConfig, flush_fn: F)
where
    F: Fn(Vec<T>) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = anyhow::Result<()>> + Send,
{
    let mut buffer = Vec::with_capacity(config.max_batch_size);
    let mut interval = tokio::time::interval(config.max_delay);
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    loop {
        tokio::select! {
            maybe_item = rx.recv() => {
                match maybe_item {
                    Some(item) => {
                        buffer.push(item);
                        if buffer.len() >= config.max_batch_size {
                            let items_to_flush = std::mem::replace(&mut buffer, Vec::with_capacity(config.max_batch_size));
                            if let Err(err) = flush_fn(items_to_flush).await {
                                error!(error = %err, "batcher failed to flush batch");
                            }
                        }
                    }
                    None => {
                        if !buffer.is_empty() {
                            if let Err(err) = flush_fn(buffer).await {
                                error!(error = %err, "batcher final flush failed");
                            }
                        }
                        break;
                    }
                }
            }
            _ = interval.tick() => {
                if !buffer.is_empty() {
                    let items_to_flush = std::mem::replace(&mut buffer, Vec::with_capacity(config.max_batch_size));
                    if let Err(err) = flush_fn(items_to_flush).await {
                        error!(error = %err, "batcher periodic flush failed");
                    }
                }
            }
        }
    }
}
