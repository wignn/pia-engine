mod auth;
mod config;
mod http;
mod proxy;
mod snapshot;
mod state;
mod tenant;
mod usage;
use crate::config::Config;
use crate::state::AppState;
use crate::tenant::TenantRegistry;
use crate::usage::UsageTracker;
use tokio::net::TcpListener;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    let cfg = Config::load();
    atlsd_observability::init_tracing("api-gateway", &cfg.log_level);

    let pool = if cfg.database_url.trim().is_empty() {
        info!("api-gateway running in zero-DB mode (Redis & memory backed)");
        None
    } else {
        match atlsd_common::db::create_resilient_pool(&cfg.database_url, 5, 1).await {
            Ok(pool) => Some(pool),
            Err(err) => {
                warn!(error = %err, "database connection failed; falling back to zero-DB mode");
                None
            }
        }
    };

    let js = if cfg.has_nats() {
        match async_nats::connect(&cfg.nats_url).await {
            Ok(client) => Some(async_nats::jetstream::new(client)),
            Err(err) => {
                warn!(error = %err, "nats connection failed; usage telemetry will be dropped");
                None
            }
        }
    } else {
        None
    };

    let redis_client = if cfg.has_redis() {
        match redis::Client::open(cfg.redis_url.clone()) {
            Ok(client) => Some(client),
            Err(err) => {
                warn!(error = %err, "invalid REDIS_URL, quota counters disabled");
                None
            }
        }
    } else {
        None
    };

    let tenant_registry = TenantRegistry::new(pool.clone());
    tenant_registry.reload().await;

    let reload_registry = tenant_registry.clone();
    tokio::spawn(async move {
        reload_registry.run_reload_loop().await;
    });

    if cfg.has_redis() {
        let sync_registry = tenant_registry.clone();
        let redis_url = cfg.redis_url.clone();
        let prefix = atlsd_common::config::get_env("REDIS_CHANNEL_PREFIX", "world-info");
        tokio::spawn(async move {
            sync_registry.run_redis_sync_loop(redis_url, prefix).await;
        });
    } else {
        warn!("api-gateway Redis config sync disabled; REDIS_URL is empty");
    }

    let usage_tracker = std::sync::Arc::new(UsageTracker::new(js, redis_client));
    let http = reqwest::Client::builder()
        .tcp_nodelay(true)
        .tcp_keepalive(Some(std::time::Duration::from_secs(30)))
        .pool_idle_timeout(Some(std::time::Duration::from_secs(90)))
        .pool_max_idle_per_host(64)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap_or_default();

    let nats_client = match async_nats::connect(&cfg.nats_url).await {
        Ok(client) => {
            info!(nats_url = %cfg.nats_url, "api-gateway connected to NATS for sub-millisecond RPC");
            Some(client)
        }
        Err(err) => {
            warn!(error = %err, nats_url = %cfg.nats_url, "api-gateway failed to connect to NATS; falling back to HTTP");
            None
        }
    };

    let state = AppState {
        config: cfg.clone(),
        http,
        nats: nats_client,
        tenant_registry,
        usage_tracker,
        internal_api_key: std::env::var("INTERNAL_API_KEY")
            .ok()
            .filter(|key| !key.trim().is_empty()),
        price_cache: std::sync::Arc::new(parking_lot::RwLock::new(None)),
        route_cache: std::sync::Arc::new(
            parking_lot::RwLock::new(std::collections::HashMap::new()),
        ),
    };

    let listener = match TcpListener::bind(&cfg.bind_addr).await {
        Ok(listener) => listener,
        Err(err) => {
            error!(error = %err, bind_addr = %cfg.bind_addr, "failed to bind api-gateway");
            std::process::exit(1);
        }
    };

    info!(bind_addr = %cfg.bind_addr, "api-gateway running with TCP_NODELAY enabled");
    let listener = atlsd_common::net::tap_nodelay(listener);
    if let Err(err) = axum::serve(listener, http::build_router(state)).await {
        error!(error = %err, "api-gateway HTTP server failed");
        std::process::exit(1);
    }
}
