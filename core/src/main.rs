mod api;
mod collector;
mod config;
mod db;
mod error;
mod html;
mod ingestion_subscriber;
mod pipeline;
mod scraper;
mod tenant;
mod ws;

use std::sync::Arc;
use std::time::Duration;
use chrono::Utc;
use serde_json::json;
use tracing::{error, info, warn};
use tracing_subscriber::{fmt, EnvFilter};
use api::state::AppState;
use api::usage_tracker::UsageTracker;
use collector::calendar::CalendarCollector;
use collector::price::{DataCollector, DataCollectorConfig};
use collector::rss::RSSCollector;
use collector::stock::StockCollector;
use collector::twitter::TwitterCollector;
use pipeline::calendar::CalendarPipeline;
use pipeline::news::NewsPipeline;
use pipeline::stock::StockPipeline;
use pipeline::twitter::TwitterPipeline;
use scraper::article::ArticleScraper;
use tenant::registry::TenantRegistry;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    let cfg = config::Config::load();

    let log_level = match cfg.log_level.to_uppercase().as_str() {
        "DEBUG" => "debug",
        "WARN" | "WARNING" => "warn",
        "ERROR" => "error",
        _ => "info",
    };

    let env_filter = EnvFilter::new(format!("core={},tower_http=debug", log_level));
    fmt()
        .json()
        .with_env_filter(env_filter)
        .with_target(true)
        .with_thread_ids(false)
        .init();

    info!(
        port = cfg.server_port,
        rss_interval = cfg.rss_fetch_sec,
        equity_interval = cfg.stock_fetch_sec,
        calendar_interval = cfg.calendar_check_sec,
        x_enabled = cfg.has_twitter(),
        redis_enabled = cfg.has_redis(),
        "core starting"
    );

    let pool = match db::create_pool(&cfg.database_url).await {
        Ok(p) => p,
        Err(e) => {
            error!(error = %e, "database connection failed");
            std::process::exit(1);
        }
    };

    let redis_client = if cfg.has_redis() {
        match redis::Client::open(cfg.redis_url.clone()) {
            Ok(client) => {
                info!(prefix = %cfg.redis_channel_prefix, "redis fanout enabled");
                Some(client)
            }
            Err(e) => {
                warn!(error = %e, "invalid REDIS_URL, continuing without redis fanout");
                None
            }
        }
    } else {
        None
    };

    let hub = ws::Hub::new(redis_client.clone(), cfg.redis_channel_prefix.clone());

    let usage_tracker = Arc::new(UsageTracker::new(pool.clone(), redis_client.clone()));

    let tenant_registry = TenantRegistry::new(pool.clone());
    tenant_registry.reload().await;
    info!("tenant registry initialized");

    let state = AppState {
        hub: hub.clone(),
        db: pool.clone(),
        config: cfg.clone(),
        tenant_registry: Some(tenant_registry.clone()),
        usage_tracker,
    };

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

    // Stats hub background task
    

    // Tenant registry background sync
    {
        let registry = tenant_registry.clone();
        let sync_rx = shutdown_tx.subscribe();
        tokio::spawn(async move {
            registry.run_sync(sync_rx).await;
        });
        info!("tenant registry sync started (60s interval)");
    }

    let timeout = Duration::from_secs(cfg.scraper_timeout);

    let rss_collector = Arc::new(RSSCollector::new(cfg.rss_max_entries, &cfg.scraper_ua, timeout));
    let article_scraper = Arc::new(ArticleScraper::new(&cfg.scraper_ua, timeout));
    let news_pipeline = Arc::new(NewsPipeline::new(
        rss_collector,
        article_scraper,
        pool.clone(),
        hub.clone(),
        redis_client.clone(),
        &cfg.redis_channel_prefix,
    ));

    if let Some(worker) = news_pipeline.build_worker() {
        tokio::spawn(async move {
            worker.run_forever().await;
        });
    }
    {
        let news_pipeline = news_pipeline.clone();
        let interval = Duration::from_secs(cfg.rss_fetch_sec);
        tokio::spawn(async move {
            pipeline::run_scheduled("news", interval, || {
                let p = news_pipeline.clone();
                async move { p.run().await }
            })
            .await;
        });
    }

    let stock_collector = Arc::new(StockCollector::new(&cfg.scraper_ua, timeout));
    let stock_pipeline = Arc::new(StockPipeline::new(
        stock_collector,
        pool.clone(),
        hub.clone(),
    ));
    {
        let stock_pipeline = stock_pipeline.clone();
        let interval = Duration::from_secs(cfg.stock_fetch_sec);
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(5)).await;
            pipeline::run_scheduled("stock", interval, || {
                let p = stock_pipeline.clone();
                async move { p.run().await }
            })
            .await;
        });
    }

    let calendar_collector = Arc::new(CalendarCollector::new(timeout));
    let calendar_pipeline = Arc::new(CalendarPipeline::new(calendar_collector, hub.clone()));
    {
        let calendar_pipeline = calendar_pipeline.clone();
        let interval = Duration::from_secs(cfg.calendar_check_sec);
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(10)).await;
            pipeline::run_scheduled("calendar", interval, || {
                let p = calendar_pipeline.clone();
                async move { p.run().await }
            })
            .await;
        });
    }

    // Twitter pipeline: merge env usernames + all tenant usernames
    if cfg.has_twitter() {
        let twitter_collector = Arc::new(TwitterCollector::new(&cfg.rsshub_url, timeout));
        let env_usernames = cfg.x_usernames.clone();
        let registry = tenant_registry.clone();

        let twitter_pipeline = Arc::new(TwitterPipeline::new(
            twitter_collector,
            hub.clone(),
            env_usernames.clone(),
            Some(registry),
        ));
        let twitter_interval = Duration::from_secs(cfg.x_poll_sec);

        info!(
            rsshub_url = %cfg.rsshub_url,
            usernames = %cfg.x_usernames,
            interval = ?twitter_interval,
            "x feed pipeline enabled (rsshub + tenant merge)"
        );

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(3)).await;
            twitter_pipeline.run(twitter_interval).await;
        });
    }

    if cfg.has_price_stream() {
        let hub = hub.clone();
        let price_cfg = DataCollectorConfig::from_app_config(&cfg);
        let reconnect_sec = cfg.tv_reconnect_sec;
        let spike_threshold_pct = cfg.tv_volatility_spike_pct;
        let spike_cooldown_sec = cfg.tv_volatility_cooldown_sec;

        info!(
            server = %cfg.tv_server,
            symbols = ?cfg.tv_symbols,
            reconnect_sec,
            spike_threshold_pct,
            spike_cooldown_sec,
            "tradingview market stream enabled"
        );

        tokio::spawn(async move {
            use std::collections::HashMap;
            use tokio::sync::Mutex;

            let last_prices = Arc::new(Mutex::new(HashMap::<String, f64>::new()));
            let last_spike_at = Arc::new(Mutex::new(HashMap::<String, i64>::new()));

            loop {
                let collector = DataCollector::new(price_cfg.clone());
                let stream_res = collector
                    .stream_forever(|tick| {
                        let hub = hub.clone();
                        let last_prices = last_prices.clone();
                        let last_spike_at = last_spike_at.clone();

                        async move {
                            let symbol = tick.symbol.clone();
                            let now_ts = Utc::now().timestamp();

                            let market_payload = json!({
                                "tick": tick,
                                "asset_type": "market",
                            });
                            let _ = hub
                                .broadcast(ws::EVENT_MARKET_TRADE, market_payload, "market_data")
                                .await;

                            let prev = {
                                let mut prices = last_prices.lock().await;
                                prices.insert(symbol.clone(), tick.price)
                            };

                            if let Some(prev_price) = prev {
                                if prev_price > 0.0 {
                                    let delta_pct = ((tick.price - prev_price).abs() / prev_price) * 100.0;
                                    let can_alert = {
                                        let spikes = last_spike_at.lock().await;
                                        spikes
                                            .get(&symbol)
                                            .map(|last| now_ts - *last >= spike_cooldown_sec as i64)
                                            .unwrap_or(true)
                                    };

                                    if delta_pct >= spike_threshold_pct && can_alert {
                                        let mut spikes = last_spike_at.lock().await;
                                        spikes.insert(symbol.clone(), now_ts);

                                        let vol_payload = json!({
                                            "symbol": symbol,
                                            "price": tick.price,
                                            "prev_price": prev_price,
                                            "delta_percent": delta_pct,
                                            "threshold_percent": spike_threshold_pct,
                                            "detected_at": Utc::now().to_rfc3339(),
                                        });

                                        let _ = hub
                                            .broadcast(
                                                ws::EVENT_GOLD_VOLATILITY_SPIKE,
                                                vol_payload,
                                                "volatility",
                                            )
                                            .await;
                                    }
                                }
                            }
                        }
                    })
                    .await;

                match stream_res {
                    Ok(()) => info!("price stream ended, reconnecting"),
                    Err(e) => error!(error = %e, "price stream failed, reconnecting"),
                }

                tokio::time::sleep(Duration::from_secs(reconnect_sec.max(1))).await;
            }
        });
    }

    // Ingestion gateway listener — subscribes to Redis channels published
    // by the external ingestion-gateway service and relays tick data into
    // the WebSocket hub so connected clients receive real-time prices.
    if cfg.has_redis() {
        let hub = hub.clone();
        let redis_url = cfg.redis_url.clone();
        tokio::spawn(async move {
            ingestion_subscriber::run(redis_url, hub).await;
        });
        info!("ingestion subscriber started (listening on ingestion:*)");
    }

    info!(port = cfg.server_port, "core running");

    if let Err(e) = api::server::start(state).await {
        error!(error = %e, "http server failed");
        std::process::exit(1);
    }

    let _ = shutdown_tx.send(true);
    info!("core stopped");
}
