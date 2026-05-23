use chrono::Utc;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info};

use crate::broker::BrokerPublisher;
use crate::config::{Config, MarketSymbolConfig};

const WORKER: &str = "index_feed";
const SOURCE: &str = "market_data";
const POLL_INTERVAL_SEC: u64 = 10;
const TOPIC: &str = "index:prices";

pub async fn run(cfg: Arc<Config>, broker: Arc<dyn BrokerPublisher>) {
    info!(worker = WORKER, "starting reference price poller");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .unwrap_or_default();

    loop {
        for symbol in cfg
            .index_feed_symbols
            .iter()
            .chain(cfg.stock_feed_symbols.iter())
        {
            if let Err(e) = poll_symbol(&client, &cfg, &*broker, symbol).await {
                error!(worker = WORKER, symbol = %symbol.public_symbol, error = %e, "failed to poll reference price");
            }
        }

        tokio::time::sleep(Duration::from_secs(POLL_INTERVAL_SEC)).await;
    }
}

async fn poll_symbol(
    client: &reqwest::Client,
    cfg: &Config,
    broker: &dyn BrokerPublisher,
    symbol: &MarketSymbolConfig,
) -> anyhow::Result<()> {
    if cfg.index_feed_http_url_template.trim().is_empty() {
        anyhow::bail!("index feed HTTP URL template not configured");
    }
    let url = cfg
        .index_feed_http_url_template
        .trim()
        .replace("{symbol}", &symbol.provider_symbol);

    let res = client.get(&url).send().await?;
    if !res.status().is_success() {
        anyhow::bail!("HTTP status error: {}", res.status());
    }

    let val: serde_json::Value = res.json().await?;

    let price = val
        .get("chart")
        .and_then(|v| v.get("result"))
        .and_then(|arr| arr.as_array())
        .and_then(|arr| arr.first())
        .and_then(|res| res.get("meta"))
        .and_then(|meta| meta.get("regularMarketPrice"))
        .and_then(|p| p.as_f64())
        .ok_or_else(|| anyhow::anyhow!("failed to parse regular market price from response"))?;

    debug!(
        worker = WORKER,
        symbol = %symbol.public_symbol,
        price = price,
        "fetched price"
    );

    let payload = json!({
        "feed": symbol.asset_type,
        "source": SOURCE,
        "symbol": symbol.public_symbol,
        "asset_type": symbol.asset_type,
        "price": price,
        "received_at": Utc::now().to_rfc3339(),
    });

    broker.publish(TOPIC, &payload.to_string()).await?;
    Ok(())
}
