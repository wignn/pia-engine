use chrono::Utc;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info};

use crate::broker::BrokerPublisher;
use crate::config::Config;

const POLL_INTERVAL_SEC: u64 = 10;
const TOPIC: &str = "yahoo:indices";

pub async fn run(_cfg: Arc<Config>, broker: Arc<dyn BrokerPublisher>) {
    info!(
        worker = "yahoo",
        "starting yahoo finance poller for SPX and DXY"
    );

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .unwrap_or_default();

    loop {
        if let Err(e) = poll_symbol(&client, &*broker, "^GSPC", "SPX").await {
            error!(worker = "yahoo", symbol = "SPX", error = %e, "failed to poll Yahoo Finance");
        }

        if let Err(e) = poll_symbol(&client, &*broker, "DX-Y.NYB", "DXY").await {
            error!(worker = "yahoo", symbol = "DXY", error = %e, "failed to poll Yahoo Finance");
        }

        tokio::time::sleep(Duration::from_secs(POLL_INTERVAL_SEC)).await;
    }
}

async fn poll_symbol(
    client: &reqwest::Client,
    broker: &dyn BrokerPublisher,
    yahoo_ticker: &str,
    normalized_symbol: &str,
) -> anyhow::Result<()> {
    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{}?interval=1m&range=1d",
        yahoo_ticker
    );

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
        .ok_or_else(|| anyhow::anyhow!("failed to parse regularMarketPrice from Yahoo response"))?;

    debug!(
        worker = "yahoo",
        ticker = yahoo_ticker,
        price = price,
        "fetched price"
    );

    let payload = json!({
        "source": "yahoo",
        "symbol": normalized_symbol,
        "price": price,
        "received_at": Utc::now().to_rfc3339(),
    });

    broker.publish(TOPIC, &payload.to_string()).await?;
    Ok(())
}
