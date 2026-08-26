mod config;
mod dlq;
mod sink;
mod writer;

use anyhow::Result;
use config::Config;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();
    let config = Config::load();
    atlsd_observability::init_tracing("sink-connector", &config.log_level);

    info!(consumer = %config.consumer_name, "macro sink connector starting");
    if let Err(err) = sink::run(config).await {
        error!(error = %err, "macro sink connector stopped");
        return Err(err);
    }
    Ok(())
}
