use std::sync::Arc;

use futures_util::StreamExt;
use serde_json::Value;
use tracing::{debug, error, info, warn};

use crate::ws;


pub async fn run(redis_url: String, hub: Arc<ws::Hub>) {
    loop {
        if let Err(e) = subscribe_loop(&redis_url, &hub).await {
            error!(error = %e, "ingestion subscriber error, reconnecting in 5s");
        }
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}

async fn subscribe_loop(redis_url: &str, hub: &Arc<ws::Hub>) -> anyhow::Result<()> {
    let client = redis::Client::open(redis_url)?;
    let mut pubsub = client.get_async_pubsub().await?;

    pubsub.psubscribe("ingestion:*").await?;
    info!("ingestion subscriber connected, listening on ingestion:*");

    loop {
        let msg = pubsub.on_message().next().await;

        let msg = match msg {
            Some(m) => m,
            None => {
                warn!("ingestion pubsub stream ended");
                return Ok(());
            }
        };

        let channel: String = msg.get_channel()?;
        let payload: String = msg.get_payload()?;

        debug!(channel = %channel, payload_len = payload.len(), "ingestion message received");

        let parsed: Value = match serde_json::from_str(&payload) {
            Ok(v) => v,
            Err(e) => {
                warn!(error = %e, channel = %channel, "failed to parse ingestion payload");
                continue;
            }
        };

        let source = parsed
            .get("source")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let symbol = parsed
            .get("symbol")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let price = parsed.get("price").and_then(|v| v.as_f64()).unwrap_or(0.0);

        if price <= 0.0 {
            continue;
        }

        let tick_data = serde_json::json!({
            "tick": {
                "symbol": normalize_symbol(source, symbol),
                "price": price,
                "bid": parsed.get("bid").and_then(|v| v.as_f64()),
                "ask": parsed.get("ask").and_then(|v| v.as_f64()),
                "volume": parsed.get("volume").and_then(|v| v.as_f64()),
                "source": source,
                "received_at": parsed.get("received_at").and_then(|v| v.as_str()),
            },
            "asset_type": "forex",
        });

        let _ = hub
            .broadcast(ws::EVENT_MARKET_TRADE, tick_data, "market_data")
            .await;
    }
}


fn normalize_symbol(source: &str, raw: &str) -> String {
    match source {
        "finnhub" => {
            if let Some((venue, pair)) = raw.split_once(':') {
                format!("{}:{}", venue, pair.replace('_', ""))
            } else {
                raw.to_string()
            }
        }
        "tiingo" => {
            format!("OANDA:{}", raw.to_uppercase())
        }
        "binance" => {
            let upper = raw.to_uppercase();
            if upper.contains(':') {
                upper
            } else {
                format!("BINANCE:{}", upper)
            }
        }
        _ => raw.to_string(),
    }
}
