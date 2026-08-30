mod config;
mod dlq;
mod workers;

use anyhow::Result;
use config::Config;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();
    let config = Config::load();
    atlsd_observability::init_tracing("sink-connector", &config.log_level);

    info!("multi-domain batch sink connector starting");

    let client = async_nats::connect(&config.nats_url).await?;
    atlsd_eventbus::nats::init_jetstream_streams(&client).await?;
    let js = async_nats::jetstream::new(client.clone());

    let pool = atlsd_common::db::create_resilient_pool(&config.database_url, 15, 2).await?;

    let mut handles = Vec::new();

    // 1. Macro Batch Worker
    if config.macro_enabled {
        let pool_macro = pool.clone();
        let js_macro = js.clone();
        let cfg_macro = config.clone();
        handles.push(tokio::spawn(async move {
            if let Err(err) = workers::macro_worker::run(pool_macro, js_macro, cfg_macro).await {
                error!(error = %err, "macro batch worker terminated with error");
            }
        }));
    }

    // 2. Usage Batch Worker (API Gateway telemetry)
    if config.usage_enabled {
        let pool_usage = pool.clone();
        let js_usage = js.clone();
        let cfg_usage = config.clone();
        handles.push(tokio::spawn(async move {
            if let Err(err) = workers::usage_worker::run(pool_usage, js_usage, cfg_usage).await {
                error!(error = %err, "usage batch worker terminated with error");
            }
        }));
    }

    // 3. News Batch Worker (Article scraping enrichment)
    if config.news_enabled {
        let pool_news = pool.clone();
        let js_news = js.clone();
        let cfg_news = config.clone();
        handles.push(tokio::spawn(async move {
            if let Err(err) = workers::news_worker::run(pool_news, js_news, cfg_news).await {
                error!(error = %err, "news batch worker terminated with error");
            }
        }));
    }

    info!(active_workers = handles.len(), "all sink workers spawned");

    if handles.is_empty() {
        info!("no sink workers enabled, idling");
        tokio::signal::ctrl_c().await?;
        return Ok(());
    }

    // Wait for any task to terminate
    let (result, _, _) = futures_util::future::select_all(handles).await;
    if let Err(join_err) = result {
        error!(error = %join_err, "a sink worker task failed");
    }

    Ok(())
}
