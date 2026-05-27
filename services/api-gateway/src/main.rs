mod auth;
mod config;
mod http;
mod proxy;
mod state;
mod tenant;
mod usage;

use tokio::net::TcpListener;
use tracing::{error, info, warn};

use crate::config::Config;
use crate::state::AppState;
use crate::tenant::TenantRegistry;
use crate::usage::UsageTracker;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    let cfg = Config::load();
    atlsd_observability::init_tracing("api-gateway", &cfg.log_level);

    let pool = match sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&cfg.database_url)
        .await
    {
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
