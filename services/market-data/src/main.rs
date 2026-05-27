mod calendar;
mod clickhouse;
mod config;
mod data_quality;
mod history;
mod http;
mod ingestion;
mod prices;
mod session;
mod spikes;
mod state;

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
    atlsd_observability::init_tracing("market-data", &cfg.log_level);

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

    let state = AppState::new(cfg.clone(), pool, clickhouse);
    calendar::hydrate(&state.db, &state.calendar).await;
    let calendar_pool = state.db.clone();
    let calendar_cache = state.calendar.clone();
    let calendar_refresh_sec = cfg.calendar_refresh_sec;
    tokio::spawn(async move {
        calendar::run_refresh(calendar_pool, calendar_cache, calendar_refresh_sec).await;
    });
    ingestion::hydrate(&state).await;

    let ingestion_state = state.clone();
    tokio::spawn(async move {
        ingestion::run(ingestion_state).await;
    });
    info!(mode = %cfg.eventbus_mode, "market-data ingestion subscriber started");

    let listener = match TcpListener::bind(&cfg.bind_addr).await {
        Ok(listener) => listener,
        Err(err) => {
            error!(error = %err, bind_addr = %cfg.bind_addr, "failed to bind market-data service");
            std::process::exit(1);
        }
    };

    info!(bind_addr = %cfg.bind_addr, "market-data service running");
    if let Err(err) = axum::serve(listener, http::build_router(state)).await {
        error!(error = %err, "market-data HTTP server failed");
        std::process::exit(1);
    }
}

pub async fn health() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "service": "market-data",
    }))
}
