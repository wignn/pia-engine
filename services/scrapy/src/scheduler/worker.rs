use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use async_nats::jetstream::{self, AckKind};
use tokio::sync::{Mutex, mpsc::Receiver};

use crate::{
    app::Config,
    models::{ScrapeJob, ScrapeResult},
    scraper::Scraper,
    storage::NatsStore,
};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

pub async fn run(
    rx: Arc<Mutex<Receiver<jetstream::Message>>>,
    store: NatsStore,
    scraper: Arc<Scraper>,
) {
    loop {
        let message = { rx.lock().await.recv().await };
        let Some(message) = message else { break };
        let fallback_id = format!("job-{}", NEXT_ID.fetch_add(1, Ordering::Relaxed));
        let job = match serde_json::from_slice::<ScrapeJob>(&message.payload)
            .map_err(anyhow::Error::from)
            .and_then(|job| job.validate(fallback_id.clone()))
        {
            Ok(job) => job,
            Err(error) => {
                let result = ScrapeResult {
                    id: fallback_id,
                    url: String::new(),
                    ok: false,
                    news: None,
                    error: Some(error.to_string()),
                };
                if publish_failure(&store, &result).await.is_ok() {
                    let _ = message.ack().await;
                }
                continue;
            }
        };

        match scraper.scrape(&job.url).await {
            Ok(news) => {
                let result = ScrapeResult {
                    id: job.id,
                    url: job.url,
                    ok: true,
                    news: Some(news),
                    error: None,
                };
                match serde_json::to_vec(&result) {
                    Ok(body) => match store.publish_result(body.into()).await {
                        Ok(()) => {
                            let _ = message.ack().await;
                        }
                        Err(_) => {
                            let _ = message
                                .ack_with(AckKind::Nak(Some(Duration::from_secs(2))))
                                .await;
                        }
                    },
                    Err(error) => tracing::error!(error = %error, "failed to encode scrape result"),
                }
            }
            Err(error) => {
                let _ = message
                    .ack_with(AckKind::Nak(Some(Duration::from_secs(2))))
                    .await;
                tracing::warn!(error = %error, "scrape failed; message scheduled for retry");
            }
        }
    }
}

async fn publish_failure(store: &NatsStore, result: &ScrapeResult) -> anyhow::Result<()> {
    store
        .publish_result(serde_json::to_vec(result)?.into())
        .await
}

pub fn worker_count(config: &Config) -> usize {
    config.workers.min(config.queue_capacity).max(1)
}
