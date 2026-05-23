use std::sync::Arc;

use tracing::{error, info, warn};

use crate::broker::{BrokerPublisher, NoopBrokerPublisher, RedisBrokerPublisher};
use crate::config::Config;
use crate::market_hours;
use crate::workers;

pub async fn run(cfg: Config) {
    info!(
        primary_fx_enabled = cfg.has_primary_fx(),
        secondary_fx_enabled = cfg.has_secondary_fx(),
        crypto_feed_enabled = cfg.crypto_feed_enabled,
        redis_enabled = cfg.has_redis(),
        market_open = market_hours::is_market_open(),
        "ingestion-gateway starting"
    );

    let broker: Arc<dyn BrokerPublisher> = if cfg.has_redis() {
        match redis::Client::open(cfg.redis_url.clone()) {
            Ok(client) => {
                info!(
                    prefix = %cfg.redis_channel_prefix,
                    "redis broker publisher initialized"
                );
                Arc::new(RedisBrokerPublisher::new(
                    client,
                    cfg.redis_channel_prefix.clone(),
                ))
            }
            Err(e) => {
                warn!(error = %e, "invalid broker URL, falling back to noop broker");
                Arc::new(NoopBrokerPublisher)
            }
        }
    } else {
        warn!("no broker URL configured, using noop broker (data will only be logged)");
        Arc::new(NoopBrokerPublisher)
    };

    let cfg = Arc::new(cfg);
    let mut worker_handles = Vec::new();

    if cfg.has_primary_fx() {
        let cfg = cfg.clone();
        let broker = broker.clone();
        let handle = tokio::spawn(async move {
            workers::primary_fx::run(cfg, broker).await;
        });
        worker_handles.push(("primary_fx", handle));
        info!(worker = "primary_fx", "worker spawned");
    } else {
        warn!("primary FX feed API key not configured, worker disabled");
    }

    if cfg.has_secondary_fx() {
        let cfg = cfg.clone();
        let broker = broker.clone();
        let handle = tokio::spawn(async move {
            workers::secondary_fx::run(cfg, broker).await;
        });
        worker_handles.push(("secondary_fx", handle));
        info!(worker = "secondary_fx", "worker spawned");
    } else {
        warn!("secondary FX feed API key not configured, worker disabled");
    }

    if cfg.crypto_feed_enabled {
        let symbols = cfg.crypto_symbols.clone();
        let cfg = cfg.clone();
        let broker = broker.clone();
        let handle = tokio::spawn(async move {
            workers::crypto_feed::run(cfg, broker).await;
        });
        worker_handles.push(("crypto_feed", handle));
        info!(symbols = ?symbols, worker = "crypto_feed", "worker spawned");
    } else {
        warn!("crypto feed disabled");
    }

    {
        let cfg = cfg.clone();
        let broker = broker.clone();
        let handle = tokio::spawn(async move {
            workers::index_feed::run(cfg, broker).await;
        });
        worker_handles.push(("index_feed", handle));
        info!(worker = "index_feed", "worker spawned");
    }

    if worker_handles.is_empty() {
        error!("no workers enabled");
        std::process::exit(1);
    }

    match tokio::signal::ctrl_c().await {
        Ok(()) => info!("shutdown signal received"),
        Err(e) => error!(error = %e, "failed to listen for shutdown signal"),
    }

    for (name, handle) in &worker_handles {
        info!(worker = name, "aborting worker");
        handle.abort();
    }

    info!("ingestion-gateway stopped");
}
