use anyhow::Result;
use async_trait::async_trait;
use atlsd_eventbus::{
    DualPublisher, EventBusMode, EventPublisher, NatsPublisher, NoopPublisher, RedisPublisher,
};
use std::sync::Arc;
use tracing::{debug, warn};

use crate::config::Config;

#[async_trait]
pub trait BrokerPublisher: Send + Sync + 'static {
    async fn publish(&self, topic: &str, payload: &str) -> Result<()>;
}

pub struct EventBusBrokerPublisher {
    publisher: Arc<dyn EventPublisher>,
}

impl EventBusBrokerPublisher {
    pub fn new(publisher: Arc<dyn EventPublisher>) -> Self {
        Self { publisher }
    }
}

#[async_trait]
impl BrokerPublisher for EventBusBrokerPublisher {
    async fn publish(&self, topic: &str, payload: &str) -> Result<()> {
        self.publisher.publish_str(topic, payload).await
    }
}

pub async fn build_broker(cfg: &Config) -> Arc<dyn BrokerPublisher> {
    let mode = EventBusMode::from_env_value(&cfg.eventbus_mode);
    let redis = build_redis(cfg);
    let nats = build_nats(cfg).await;

    let publisher: Arc<dyn EventPublisher> = match mode {
        EventBusMode::Redis => redis.unwrap_or_else(|| Arc::new(NoopPublisher)),
        EventBusMode::Nats => nats.unwrap_or_else(|| Arc::new(NoopPublisher)),
        EventBusMode::Dual => match (redis, nats) {
            (Some(redis), Some(nats)) => Arc::new(DualPublisher::new(redis, nats)),
            (Some(redis), None) => redis,
            (None, Some(nats)) => nats,
            (None, None) => Arc::new(NoopPublisher),
        },
        EventBusMode::Noop => Arc::new(NoopPublisher),
    };

    Arc::new(EventBusBrokerPublisher::new(publisher))
}

fn build_redis(cfg: &Config) -> Option<Arc<dyn EventPublisher>> {
    if !cfg.has_redis() {
        warn!("Redis eventbus disabled; REDIS_URL is empty");
        return None;
    }

    match redis::Client::open(cfg.redis_url.clone()) {
        Ok(client) => {
            debug!(prefix = %cfg.redis_channel_prefix, "Redis eventbus publisher initialized");
            Some(Arc::new(RedisPublisher::new(
                client,
                cfg.redis_channel_prefix.clone(),
            )))
        }
        Err(err) => {
            warn!(error = %err, "invalid Redis URL for eventbus");
            None
        }
    }
}

async fn build_nats(cfg: &Config) -> Option<Arc<dyn EventPublisher>> {
    match NatsPublisher::connect(&cfg.nats_url).await {
        Ok(publisher) => {
            debug!(url = %cfg.nats_url, "NATS eventbus publisher initialized");
            Some(Arc::new(publisher))
        }
        Err(err) => {
            warn!(error = %err, url = %cfg.nats_url, "NATS eventbus publisher unavailable");
            None
        }
    }
}
