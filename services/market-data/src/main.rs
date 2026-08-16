mod alert_notifier;
mod alerts;
mod batcher;
mod calendar;
mod candle_engine;
mod clickhouse;
mod config;
mod cot;
mod data_quality;
mod deadletter;
mod economic;
mod energy;
mod fear_greed;
mod history;
mod http;
mod ingestion;
mod institutional;
mod options;
mod prices;
mod rates;
mod session;
mod spikes;
mod state;

use axum::Json;
use serde_json::{json, Value};
use std::{sync::Arc, time::Duration};
use tokio::{net::TcpListener, sync::mpsc};
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

    let (tick_tx, tick_rx) = if clickhouse.is_some() {
        let (tx, rx) = mpsc::channel(10_000);
        (Some(tx), Some(rx))
    } else {
        (None, None)
    };

    if let (Some(ch), Some(rx)) = (clickhouse.clone(), tick_rx) {
        let dlq_pool = pool.clone();
        tokio::spawn(async move {
            batcher::run_batcher(
                rx,
                batcher::BatcherConfig {
                    max_batch_size: 1000,
                    max_delay: Duration::from_secs(1),
                    ..batcher::BatcherConfig::default()
                },
                move |batch| {
                    let client = ch.clone();
                    async move { client.insert_price_ticks_batch(&batch).await }
                },
                move |batch, err| {
                    let pool = dlq_pool.clone();
                    async move {
                        deadletter::record_tick_batch(&pool, &batch, &err).await;
                    }
                },
            )
            .await;
        });
    }

    let metrics = std::sync::Arc::new(atlsd_observability::MetricsRegistry::new());
    let candle_enabled = matches!(
        atlsd_eventbus::EventBusMode::from_env_value(&cfg.eventbus_mode),
        atlsd_eventbus::EventBusMode::Nats | atlsd_eventbus::EventBusMode::Dual
    );
    let (candle, candle_rx) = if candle_enabled {
        let grace_secs: u64 = atlsd_common::config::get_env("CANDLE_GRACE_SEC", "5")
            .parse()
            .unwrap_or(5);
        let (handle, rx) =
            candle_engine::CandleEngineHandle::new(grace_secs, Some(metrics.clone()));
        (Some(handle), Some(rx))
    } else {
        (None, None)
    };

    if let (Some(handle), Some(rx)) = (candle.clone(), candle_rx) {
        if let Some(clickhouse) = clickhouse.as_ref() {
            candle_engine::bootstrap_from_ticks(&handle, clickhouse).await;
        }
        let collector_handle = handle.clone();
        tokio::spawn(async move {
            candle_engine::run_collector(collector_handle).await;
        });
        let nats_url = cfg.nats_url.clone();
        let publisher_metrics = metrics.clone();
        tokio::spawn(async move {
            if let Err(err) =
                candle_engine::run_publisher(rx, &nats_url, Some(publisher_metrics)).await
            {
                error!(error = %err, "candle event publisher exited");
            }
        });
    }

    let state = AppState::new(cfg.clone(), pool, clickhouse, tick_tx, candle, metrics);
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

    let alert_state = state.clone();
    tokio::spawn(async move {
        alert_notifier::run(alert_state).await;
    });

    if cfg.has_fred() {
        let econ_cfg = cfg.clone();
        let econ_pool = state.db.clone();
        tokio::spawn(async move {
            economic::run_sync(econ_cfg, econ_pool).await;
        });
        info!("economic data sync (FRED) enabled");

        let rates_cfg = cfg.clone();
        let rates_pool = state.db.clone();
        tokio::spawn(async move {
            rates::run_rates_sync(rates_cfg, rates_pool).await;
        });
        info!("rates data sync (FRED) enabled");
    }

    if cfg.has_eia() {
        let eia_cfg = cfg.clone();
        let eia_pool = state.db.clone();
        tokio::spawn(async move {
            energy::run_energy_sync(eia_cfg, eia_pool).await;
        });
        info!("energy data sync (EIA) enabled");
    }

    let cot_cfg = cfg.clone();
    let cot_pool = state.db.clone();
    tokio::spawn(async move {
        cot::run_cot_sync(cot_cfg, cot_pool).await;
    });
    info!("CFTC COT positioning data sync enabled");

    let fg_cfg = cfg.clone();
    let fg_pool = state.db.clone();
    let fg_clickhouse = state.clickhouse.clone();
    tokio::spawn(async move {
        fear_greed::run_fear_greed_sync(fg_cfg, fg_pool, fg_clickhouse).await;
    });
    info!("Fear & Greed risk regime index sync enabled");

    let institutional_cfg = cfg.clone();
    let institutional_pool = state.db.clone();
    tokio::spawn(async move {
        institutional::run_sync(institutional_cfg, institutional_pool).await;
    });
    info!("institutional market data sync enabled");

    let options_state = state.clone();
    tokio::spawn(async move {
        options::run_options_subscriber(options_state).await;
    });
    info!("options data subscriber enabled");

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
