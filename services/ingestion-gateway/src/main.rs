mod broker;
mod config;
mod market_hours;
mod runtime;
mod workers;

use config::Config;

#[tokio::main]
async fn main() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let _ = dotenvy::dotenv();

    let cfg = Config::load();

    atlsd_observability::init_tracing("ingestion_gateway", &cfg.log_level);

    runtime::run(cfg).await;
}
