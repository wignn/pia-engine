use atlsd_eventbus::{
    DualPublisher, EventBusMode, EventPublisher, NatsPublisher, NoopPublisher, RedisPublisher,
};
use std::sync::Arc;
use tracing::{debug, warn};

use crate::config::Config;
use crate::workers::reconnect::ReconnectPolicy;

pub async fn build_broker(cfg: &Config) -> Arc<dyn EventPublisher> {
    let mode = EventBusMode::from_env_value(&cfg.eventbus_mode);
    let redis = build_redis(cfg);
    let nats = match mode {
        EventBusMode::Nats | EventBusMode::Dual => build_nats(cfg).await,
        EventBusMode::Redis | EventBusMode::Noop => None,
    };

    match mode {
        EventBusMode::Redis => redis.unwrap_or_else(|| Arc::new(NoopPublisher)),
        EventBusMode::Nats => nats.unwrap_or_else(|| Arc::new(NoopPublisher)),
        EventBusMode::Dual => match (redis, nats) {
            (Some(redis), Some(nats)) => Arc::new(DualPublisher::new(redis, nats)),
            (Some(redis), None) => redis,
            (None, Some(nats)) => nats,
            (None, None) => Arc::new(NoopPublisher),
        },
        EventBusMode::Noop => Arc::new(NoopPublisher),
    }
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
    let mut backoff = ReconnectPolicy::new(cfg.reconnect_base_sec, cfg.reconnect_max_sec);

    loop {
        match NatsPublisher::connect(&cfg.nats_url).await {
            Ok(publisher) => {
                debug!(url = %cfg.nats_url, "NATS eventbus publisher initialized");
                return Some(Arc::new(publisher));
            }
            Err(err) => {
                let delay = backoff.next_delay();
                warn!(
                    error = %err,
                    url = %cfg.nats_url,
                    retry_secs = delay.as_secs(),
                    "NATS eventbus publisher unavailable, retrying"
                );
                tokio::time::sleep(delay).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::time::Duration;

    fn test_config(nats_url: String) -> Config {
        Config {
            primary_fx_api_key: String::new(),
            secondary_fx_api_key: String::new(),
            primary_fx_ws_url: String::new(),
            crypto_feed_ws_url: String::new(),
            tradingview_quote_url_template: String::new(),
            primary_fx_symbols: Vec::new(),
            secondary_fx_symbols: Vec::new(),
            index_feed_symbols: Vec::new(),
            stock_feed_symbols: Vec::new(),
            crypto_symbols: Vec::new(),
            crypto_feed_enabled: false,
            redis_url: String::new(),
            redis_channel_prefix: "ingestion".to_string(),
            eventbus_mode: "nats".to_string(),
            nats_url,
            reconnect_base_sec: 1,
            reconnect_max_sec: 1,
            market_check_interval_sec: 30,
            health_bind_addr: "127.0.0.1:0".to_string(),
            health_stale_after_sec: 180,
            log_level: "INFO".to_string(),
        }
    }

    #[tokio::test]
    async fn build_nats_retries_instead_of_returning_none() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            while let Ok((_socket, _)) = listener.accept().await {
                // Drop the socket immediately so NATS connect fails quickly.
            }
        });
        let cfg = test_config(format!("nats://{addr}"));

        let result = tokio::time::timeout(Duration::from_millis(500), build_nats(&cfg)).await;

        assert!(
            result.is_err(),
            "build_nats should keep retrying startup failures"
        );
    }

    #[tokio::test]
    async fn build_broker_does_not_wait_for_nats_in_redis_mode() {
        let mut cfg = test_config("nats://127.0.0.1:1".to_string());
        cfg.eventbus_mode = "redis".to_string();

        let result = tokio::time::timeout(Duration::from_millis(500), build_broker(&cfg)).await;

        assert!(result.is_ok(), "Redis mode should not initialize NATS");
    }
}
