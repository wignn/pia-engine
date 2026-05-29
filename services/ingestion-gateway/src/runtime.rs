use std::sync::Arc;

use tracing::{error, info, warn};

use crate::broker::{self, BrokerPublisher};
use crate::config::Config;
use crate::health::HealthRegistry;
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

    let health = HealthRegistry::new((cfg.health_stale_after_sec * 1_000) as i64);
    health.register("primary_fx", cfg.has_primary_fx()).await;
    health
        .register("secondary_fx", cfg.has_secondary_fx())
        .await;
    health
        .register("crypto_feed", cfg.crypto_feed_enabled)
        .await;

    let health_bind_addr = cfg.health_bind_addr.clone();
    let health_server = tokio::spawn(crate::health::serve(health_bind_addr, health.clone()));

    let broker: Arc<dyn BrokerPublisher> = broker::build_broker(&cfg).await;

    let cfg = Arc::new(cfg);
    let mut worker_handles = Vec::new();

    if cfg.has_primary_fx() {
        let cfg = cfg.clone();
        let broker = broker.clone();
        let health = health.clone();
        let handle = tokio::spawn(async move {
            workers::primary_fx::run(cfg, broker, health).await;
        });
        worker_handles.push(("primary_fx", handle));
        info!(worker = "primary_fx", "worker spawned");
    } else {
        warn!("primary FX feed API key not configured, worker disabled");
    }

    if cfg.has_secondary_fx() {
        let cfg = cfg.clone();
        let broker = broker.clone();
        let health = health.clone();
        let handle = tokio::spawn(async move {
            workers::secondary_fx::run(cfg, broker, health).await;
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
        let health = health.clone();
        let handle = tokio::spawn(async move {
            workers::crypto_feed::run(cfg, broker, health).await;
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

    health_server.abort();

    info!("ingestion-gateway stopped");
}
