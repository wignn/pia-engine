pub mod worker;

use crate::{app::Config, error::Result, scraper::Scraper, storage::NatsStore};
use std::{sync::Arc, time::Duration};
use tokio::sync::{Mutex, mpsc};

pub struct Scheduler {
    store: NatsStore,
    scraper: Arc<Scraper>,
    config: Config,
}

impl Scheduler {
    pub fn new(store: NatsStore, scraper: Arc<Scraper>, config: Config) -> Self {
        Self {
            store,
            scraper,
            config,
        }
    }

    pub async fn run(self) -> Result<()> {
        let (tx, rx) = mpsc::channel(self.config.queue_capacity);
        let intake = tokio::spawn(intake(self.store.clone(), tx));
        let shared_rx = Arc::new(Mutex::new(rx));
        let workers = (0..worker::worker_count(&self.config))
            .map(|_| {
                tokio::spawn(worker::run(
                    shared_rx.clone(),
                    self.store.clone(),
                    self.scraper.clone(),
                ))
            })
            .collect::<Vec<_>>();

        tokio::signal::ctrl_c().await?;
        intake.abort();
        for task in workers {
            task.abort();
            let _ = task.await;
        }
        self.store.flush().await
    }
}

async fn intake(
    store: NatsStore,
    tx: mpsc::Sender<async_nats::jetstream::Message>,
) -> anyhow::Result<()> {
    loop {
        let permit = tx
            .reserve()
            .await
            .map_err(|_| anyhow::anyhow!("worker queue closed"))?;
        match store.next_message().await? {
            Some(message) => permit.send(message),
            None => drop(permit),
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
