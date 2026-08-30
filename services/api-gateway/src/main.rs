mod auth;
mod config;
mod http;
mod proxy;
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

    let pool = match atlsd_common::db::create_resilient_pool(&cfg.database_url, 5, 1).await {
        Ok(pool) => pool,
        Err(err) => {
            error!(error = %err, "database connection failed");
            std::process::exit(1);
        }
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

    let usage_tracker = std::sync::Arc::new(UsageTracker::new(pool.clone(), redis_client));
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap_or_default();

    let state = AppState {
        config: cfg.clone(),
        http,
        tenant_registry,
        usage_tracker,
        internal_api_key: std::env::var("INTERNAL_API_KEY")
            .ok()
            .filter(|key| !key.trim().is_empty()),
    };

    let listener = match TcpListener::bind(&cfg.bind_addr).await {
        Ok(listener) => listener,
        Err(err) => {
            error!(error = %err, bind_addr = %cfg.bind_addr, "failed to bind api-gateway");
            std::process::exit(1);
        }
    };

    info!(bind_addr = %cfg.bind_addr, "api-gateway running");
    if let Err(err) = axum::serve(listener, http::build_router(state)).await {
        error!(error = %err, "api-gateway HTTP server failed");
        std::process::exit(1);
    }
}
