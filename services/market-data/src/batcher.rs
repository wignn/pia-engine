use std::time::Duration;

use tokio::sync::mpsc;
use tracing::{error, warn};

pub struct BatcherConfig {
    pub max_batch_size: usize,
    pub max_delay: Duration,
    pub max_retries: u32,
    pub retry_base_delay: Duration,
}

impl Default for BatcherConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 1000,
            max_delay: Duration::from_secs(1),
            max_retries: 5,
            retry_base_delay: Duration::from_secs(1),
        }
    }
}

pub async fn run_batcher<T, F, Fut, D, DFut>(
    mut rx: mpsc::Receiver<T>,
    config: BatcherConfig,
    flush_fn: F,
    dead_letter_fn: D,
) where
    T: Clone,
    F: Fn(Vec<T>) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = anyhow::Result<()>> + Send,
    D: Fn(Vec<T>, anyhow::Error) -> DFut + Send + Sync + 'static,
    DFut: std::future::Future<Output = ()> + Send,
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
                            flush_with_dead_letter(items_to_flush, &config, &flush_fn, &dead_letter_fn).await;
                        }
                    }
                    None => {
                        if !buffer.is_empty() {
                            flush_with_dead_letter(buffer, &config, &flush_fn, &dead_letter_fn).await;
                        }
                        break;
                    }
                }
            }
            _ = interval.tick() => {
                if !buffer.is_empty() {
                    let items_to_flush = std::mem::replace(&mut buffer, Vec::with_capacity(config.max_batch_size));
                    flush_with_dead_letter(items_to_flush, &config, &flush_fn, &dead_letter_fn).await;
                }
            }
        }
    }
}

async fn flush_with_dead_letter<T, F, Fut, D, DFut>(
    items: Vec<T>,
    config: &BatcherConfig,
    flush_fn: &F,
    dead_letter_fn: &D,
) where
    T: Clone,
    F: Fn(Vec<T>) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = anyhow::Result<()>> + Send,
    D: Fn(Vec<T>, anyhow::Error) -> DFut + Send + Sync + 'static,
    DFut: std::future::Future<Output = ()> + Send,
{
    let batch_size = items.len();
    let mut last_err = None;

    for attempt in 0..=config.max_retries {
        if attempt > 0 {
            let delay = config.retry_base_delay * (1u32 << attempt.min(5));
            warn!(
                attempt,
                max_retries = config.max_retries,
                batch_size,
                retry_secs = delay.as_secs(),
                "batcher flush failed, retrying"
            );
            tokio::time::sleep(delay).await;
        }

        match flush_fn(items.clone()).await {
            Ok(()) => return,
            Err(err) => last_err = Some(err),
        }
    }

    let err = last_err.unwrap_or_else(|| anyhow::anyhow!("batcher flush failed without error"));
    error!(
        batch_size,
        attempts = config.max_retries + 1,
        error = %err,
        "batcher exhausted retries, sending batch to dead letter"
    );
    dead_letter_fn(items, err).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    fn fast_config() -> BatcherConfig {
        BatcherConfig {
            max_batch_size: 2,
            max_delay: Duration::from_secs(30),
            max_retries: 3,
            retry_base_delay: Duration::from_millis(1),
        }
    }

    #[tokio::test]
    async fn retries_then_succeeds_without_dead_letter() {
        let (tx, rx) = mpsc::channel(10);
        let failures = Arc::new(AtomicU32::new(2));
        let dead_lettered = Arc::new(AtomicU32::new(0));

        let failures_clone = failures.clone();
        let dead_clone = dead_lettered.clone();
        tokio::spawn(async move {
            run_batcher(
                rx,
                fast_config(),
                move |batch: Vec<u32>| {
                    let failures = failures_clone.clone();
                    async move {
                        let remaining = failures.load(Ordering::SeqCst);
                        if remaining > 0 {
                            failures.store(remaining - 1, Ordering::SeqCst);
                            anyhow::bail!("transient failure for {} items", batch.len());
                        }
                        Ok(())
                    }
                },
                move |_: Vec<u32>, _: anyhow::Error| {
                    let dead = dead_clone.clone();
                    async move {
                        dead.fetch_add(1, Ordering::SeqCst);
                    }
                },
            )
            .await;
        });

        tx.send(1).await.unwrap();
        tx.send(2).await.unwrap();
        tx.send(3).await.unwrap();
        drop(tx);
        tokio::time::sleep(Duration::from_millis(200)).await;

        assert_eq!(
            dead_lettered.load(Ordering::SeqCst),
            0,
            "should not dead-letter after successful retry"
        );
    }

    #[tokio::test]
    async fn dead_letters_after_exhausting_retries() {
        let (tx, rx) = mpsc::channel(10);
        let dead_lettered = Arc::new(AtomicU32::new(0));

        let dead_clone = dead_lettered.clone();
        tokio::spawn(async move {
            run_batcher(
                rx,
                fast_config(),
                |_batch: Vec<u32>| async { anyhow::bail!("clickhouse down") },
                move |batch: Vec<u32>, err: anyhow::Error| {
                    let dead = dead_clone.clone();
                    async move {
                        assert_eq!(batch, vec![1, 2]);
                        assert!(!err.to_string().is_empty());
                        dead.fetch_add(1, Ordering::SeqCst);
                    }
                },
            )
            .await;
        });

        tx.send(1).await.unwrap();
        tx.send(2).await.unwrap();
        drop(tx);
        tokio::time::sleep(Duration::from_millis(200)).await;

        assert_eq!(
            dead_lettered.load(Ordering::SeqCst),
            1,
            "exhausted batch must reach dead letter"
        );
    }
}
