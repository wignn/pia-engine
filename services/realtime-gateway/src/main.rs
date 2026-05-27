mod client;
mod config;
mod http;
mod hub;
mod nats_subscriber;
mod redis_subscriber;
mod state;
mod streams;

use axum::Json;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{error, info, warn};

use crate::config::Config;
use crate::hub::Hub;
use crate::state::AppState;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    let cfg = Config::load();
    atlsd_observability::init_tracing("realtime-gateway", &cfg.log_level);

    let hub = Hub::new(None, cfg.redis_channel_prefix.clone());
    let state = AppState {
        config: cfg.clone(),
        hub: hub.clone(),
        ticket_store: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
    };

    if cfg.has_redis() {
        let redis_url = cfg.redis_url.clone();
        let prefix = cfg.redis_channel_prefix.clone();
        let redis_hub = hub.clone();
        tokio::spawn(async move {
            redis_subscriber::run(redis_url, prefix, redis_hub).await;
        });
        info!("realtime gateway Redis subscriber started");
    } else {
        warn!("realtime Redis subscriber disabled; REDIS_URL is empty or subscribe disabled");
    }

    let nats_cfg = cfg.clone();
    let nats_hub = hub.clone();
    tokio::spawn(async move {
        nats_subscriber::run(nats_cfg, nats_hub).await;
    });

    let listener = match TcpListener::bind(&cfg.bind_addr).await {
        Ok(listener) => listener,
        Err(err) => {
            error!(error = %err, bind_addr = %cfg.bind_addr, "failed to bind realtime gateway");
            std::process::exit(1);
        }
    };

    info!(bind_addr = %cfg.bind_addr, "realtime gateway running");
    if let Err(err) = axum::serve(listener, http::build_router(state)).await {
        error!(error = %err, "realtime gateway HTTP server failed");
        std::process::exit(1);
    }
}

pub async fn health() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "service": "realtime-gateway",
    }))
}
