mod config;
mod http;
mod news;
mod pipeline;
mod realtime;
mod state;

use axum::Json;
use serde_json::{json, Value};
use tokio::net::TcpListener;
use tracing::{error, info};

use crate::config::Config;
use crate::state::AppState;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    let cfg = Config::load();
    atlsd_observability::init_tracing("news-service", &cfg.log_level);

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

    if cfg.run_pipelines {
        let pipeline_cfg = cfg.clone();
        let pipeline_pool = pool.clone();
        tokio::spawn(async move {
            pipeline::run(pipeline_cfg, pipeline_pool).await;
        });
    }

    let realtime_cfg = cfg.clone();
    let realtime_pool = pool.clone();
    tokio::spawn(async move {
        realtime::run(realtime_cfg, realtime_pool).await;
    });

    let state = AppState { db: pool };
    let listener = match TcpListener::bind(&cfg.bind_addr).await {
        Ok(listener) => listener,
        Err(err) => {
            error!(error = %err, bind_addr = %cfg.bind_addr, "failed to bind news-service");
            std::process::exit(1);
        }
    };

    info!(bind_addr = %cfg.bind_addr, "news-service running");
    if let Err(err) = axum::serve(listener, http::build_router(state)).await {
        error!(error = %err, "news-service HTTP server failed");
        std::process::exit(1);
    }
}

pub async fn health() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "service": "news-service",
    }))
}
