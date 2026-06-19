use atlsd_eventbus::{subjects, EventPublisher};
use chrono::Utc;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info};

use crate::config::{Config, MarketSymbolConfig};
use crate::workers::tradingview;

const WORKER: &str = "index_feed";
const SOURCE: &str = "market_data";
const POLL_INTERVAL_SEC: u64 = 10;
const TOPIC: &str = subjects::MD_RAW_INDEX_QUOTES_V1;

pub async fn run(cfg: Arc<Config>, broker: Arc<dyn EventPublisher>) {
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
    broker: &dyn EventPublisher,
    symbol: &MarketSymbolConfig,
) -> anyhow::Result<()> {
    let template = if cfg.tradingview_quote_url_template.trim().is_empty() {
        ""
    } else {
        cfg.tradingview_quote_url_template.trim()
    };

    let tv_symbol = tradingview::symbol_for(
        &symbol.provider_symbol,
        &symbol.public_symbol,
        &symbol.asset_type,
    );
    let price = tradingview::fetch_quote(client, template, &tv_symbol).await?;

    debug!(
        worker = WORKER,
        symbol = %symbol.public_symbol,
        provider_symbol = %tv_symbol,
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

    broker.publish_str(TOPIC, &payload.to_string()).await?;
    Ok(())
}
