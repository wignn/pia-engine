mod broker;
mod config;
mod market_hours;
mod workers;

use std::sync::Arc;

use tracing::{error, info, warn};
use tracing_subscriber::{fmt, EnvFilter};

use broker::{BrokerPublisher, NoopBrokerPublisher, RedisBrokerPublisher};
use config::Config;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    let cfg = Config::load();

    let log_level = match cfg.log_level.to_uppercase().as_str() {
        "DEBUG" => "debug",
        "WARN" | "WARNING" => "warn",
        "ERROR" => "error",
        "TRACE" => "trace",
        _ => "info",
    };

    let env_filter = EnvFilter::new(format!("ingestion_gateway={}", log_level));
    fmt()
        .json()
        .with_env_filter(env_filter)
        .with_target(true)
        .with_thread_ids(false)
        .init();

    info!(
        finnhub_enabled = cfg.has_finnhub(),
        tiingo_enabled = cfg.has_tiingo(),
        binance_enabled = cfg.binance_enabled,
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
                Arc::new(RedisBrokerPublisher::new(client, cfg.redis_channel_prefix.clone()))
            }
            Err(e) => {
                warn!(error = %e, "invalid REDIS_URL, falling back to noop broker");
                Arc::new(NoopBrokerPublisher)
            }
        }
    } else {
        warn!("no REDIS_URL configured, using noop broker (data will only be logged)");
        Arc::new(NoopBrokerPublisher)
    };

    let cfg = Arc::new(cfg);

    let mut worker_handles = Vec::new();

    if cfg.has_finnhub() {
        let cfg = cfg.clone();
        let broker = broker.clone();
        let handle = tokio::spawn(async move {
            workers::finnhub::run(cfg, broker).await;
        });
        worker_handles.push(("finnhub", handle));
        info!("finnhub worker spawned");
    } else {
        warn!("FINNHUB_API_KEY not set, finnhub worker disabled");
    }

    if cfg.has_tiingo() {
        let cfg = cfg.clone();
        let broker = broker.clone();
        let handle = tokio::spawn(async move {
            workers::tiingo::run(cfg, broker).await;
        });
        worker_handles.push(("tiingo", handle));
        info!("tiingo worker spawned");
    } else {
        warn!("TIINGO_API_KEY not set, tiingo worker disabled");
    }

    if cfg.binance_enabled {
        let binance_cfg = cfg.clone();
        let broker = broker.clone();
        let handle = tokio::spawn(async move {
            workers::binance::run(binance_cfg, broker).await;
        });
        worker_handles.push(("binance", handle));
        info!(
            symbols = ?cfg.binance_symbols,
            "binance worker spawned (no API key needed)"
        );
    } else {
        warn!("BINANCE_ENABLED=false, binance worker disabled");
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
