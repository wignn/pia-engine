use std::{env, sync::Arc, time::Duration};

use crate::{error::Result, scheduler::Scheduler, scraper::Scraper, storage::NatsStore};

#[derive(Debug, Clone)]
pub struct Config {
    pub nats_url: String,
    pub input_subject: String,
    pub output_subject: String,
    pub jobs_stream: String,
    pub results_stream: String,
    pub consumer_name: String,
    pub queue_capacity: usize,
    pub workers: usize,
    pub request_timeout: Duration,
    pub max_body_bytes: usize,
    pub max_deliver: i64,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let queue_capacity = env_usize("SCRAPE_QUEUE_CAPACITY", 32)?.max(1);
        Ok(Self {
            nats_url: env_string("NATS_URL", "nats://127.0.0.1:4222"),
            input_subject: env_string("SCRAPE_INPUT_SUBJECT", "scrape.jobs"),
            output_subject: env_string("SCRAPE_OUTPUT_SUBJECT", "scrape.results"),
            jobs_stream: env_string("SCRAPE_JOBS_STREAM", "SCRAPE_JOBS"),
            results_stream: env_string("SCRAPE_RESULTS_STREAM", "SCRAPE_RESULTS"),
            consumer_name: env_string("SCRAPE_CONSUMER", "scraper-worker"),
            queue_capacity,
            workers: env_usize("SCRAPE_WORKERS", 4)?.clamp(1, queue_capacity),
            request_timeout: Duration::from_secs(env_u64("SCRAPE_REQUEST_TIMEOUT_SECS", 20)?),
            max_body_bytes: env_usize("SCRAPE_MAX_BODY_BYTES", 4 * 1024 * 1024)?,
            max_deliver: env_i64("SCRAPE_MAX_DELIVER", 5)?.max(1),
        })
    }
}

pub struct App {
    config: Config,
}

impl App {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn run(self) -> Result<()> {
        let store = NatsStore::connect(&self.config).await?;
        let scraper = Arc::new(Scraper::new(
            self.config.request_timeout,
            self.config.max_body_bytes,
        )?);
        Scheduler::new(store, scraper, self.config).run().await
    }
}

fn env_string(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_owned())
}

fn env_usize(name: &str, default: usize) -> Result<usize> {
    env_string(name, &default.to_string())
        .parse()
        .map_err(|e| anyhow::anyhow!("{name} must be a positive integer: {e}"))
}

fn env_u64(name: &str, default: u64) -> Result<u64> {
    env_string(name, &default.to_string())
        .parse()
        .map_err(|e| anyhow::anyhow!("{name} must be an integer: {e}"))
}

fn env_i64(name: &str, default: i64) -> Result<i64> {
    env_string(name, &default.to_string())
        .parse()
        .map_err(|e| anyhow::anyhow!("{name} must be an integer: {e}"))
}
