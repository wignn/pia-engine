mod api;
mod config;
mod models;
mod billing;
mod sync;

use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    let cfg = config::Config::load();

    let env_filter = EnvFilter::new(format!("control_plane={},tower_http=debug", cfg.log_level));
    fmt()
        .json()
        .with_env_filter(env_filter)
        .with_target(true)
        .with_thread_ids(false)
        .init();

    info!(port = cfg.port, "control-plane starting");

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(&cfg.database_url)
        .await
        .expect("failed to connect to database");

    info!("database connected");

    let redis_client = if !cfg.redis_url.is_empty() {
        match redis::Client::open(cfg.redis_url.clone()) {
            Ok(c) => {
                info!("redis connected");
                Some(c)
            }
            Err(e) => {
                tracing::warn!(error = %e, "redis unavailable, running without sync");
                None
            }
        }
    } else {
        None
    };

    let state = api::AppState {
        db: pool,
        config: cfg.clone(),
        redis: redis_client,
    };

    info!(port = cfg.port, "control-plane running");

    if let Err(e) = api::server::start(state).await {
        tracing::error!(error = %e, "control-plane server failed");
        std::process::exit(1);
    }
}
