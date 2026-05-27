mod clickhouse;
mod config;
mod http;
mod sentiment;
mod state;
mod why_move;

use axum::Json;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{error, info};

use crate::clickhouse::ClickHouseClient;
use crate::config::Config;
use crate::state::AppState;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    let cfg = Config::load();
    atlsd_observability::init_tracing("intelligence-service", &cfg.log_level);

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

    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap_or_default();
    let clickhouse = if cfg.has_clickhouse() {
        Some(Arc::new(ClickHouseClient::new(
            cfg.clickhouse_url.clone(),
            cfg.clickhouse_database.clone(),
            cfg.clickhouse_user.clone(),
            cfg.clickhouse_password.clone(),
        )))
    } else {
        None
    };
    let state = AppState {
        config: cfg.clone(),
        db: pool,
        http,
        clickhouse,
    };

    let listener = match TcpListener::bind(&cfg.bind_addr).await {
        Ok(listener) => listener,
        Err(err) => {
            error!(error = %err, bind_addr = %cfg.bind_addr, "failed to bind intelligence-service");
            std::process::exit(1);
        }
    };

    info!(bind_addr = %cfg.bind_addr, "intelligence-service running");
    if let Err(err) = axum::serve(listener, http::build_router(state)).await {
        error!(error = %err, "intelligence-service HTTP server failed");
        std::process::exit(1);
    }
}

pub async fn health() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "service": "intelligence-service",
    }))
}
