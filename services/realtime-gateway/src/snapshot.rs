use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use serde_json::{json, Value};
use tracing::{info, warn};

use crate::hub::{ClientId, Hub};

pub struct Snapshot {
    http: reqwest::Client,
    market_data_url: String,
}

impl Snapshot {
    pub fn new(market_data_url: String) -> Self {
        let internal_api_key = std::env::var("INTERNAL_API_KEY")
            .ok()
            .filter(|key| !key.trim().is_empty());
        let mut builder = reqwest::Client::builder().timeout(Duration::from_secs(2));
        if let Some(key) = internal_api_key {
            let mut headers = reqwest::header::HeaderMap::new();
            if let Ok(value) = reqwest::header::HeaderValue::from_str(&key) {
                headers.insert("x-internal-api-key", value);
            }
            builder = builder.default_headers(headers);
        }
        Self {
            http: builder.build().expect("reqwest client"),
            market_data_url,
        }
    }
}

/// True when the subscribed streams should receive a market snapshot.
pub fn wants_market_snapshot(streams: &HashSet<String>) -> bool {
    streams
        .iter()
        .any(|stream| stream == "market_data" || stream == "all" || stream.starts_with("market."))
}

pub async fn send_snapshot(snapshot: &Arc<Snapshot>, hub: &Arc<Hub>, client_id: ClientId) {
    let url = format!("{}/api/v1/market/prices", snapshot.market_data_url);
    let fetched = snapshot.http.get(&url).send().await;

    let response = match fetched {
        Ok(response) if response.status().is_success() => response,
        Ok(response) => {
            warn!(
                status = %response.status(),
                "market snapshot skipped: market-data returned error"
            );
            return;
        }
        Err(err) => {
            warn!(error = %err, "market snapshot skipped: market-data unreachable");
            return;
        }
    };

    let prices: Value = match response.json().await {
        Ok(value) => value,
        Err(err) => {
            warn!(error = %err, "market snapshot skipped: invalid payload");
            return;
        }
    };

    let count = prices
        .get("items")
        .and_then(Value::as_array)
        .map(|items| items.len())
        .unwrap_or(0);
    let payload = serde_json::to_vec(&shape_snapshot(prices)).unwrap_or_default();
    if hub.push_to_client(client_id, payload).await {
        info!(
            client = client_id,
            symbols = count,
            "market snapshot delivered"
        );
    }
}

fn shape_snapshot(prices: Value) -> Value {
    json!({
        "event": "market.snapshot",
        "data": prices,
        "channel": "market_data",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(values: &[&str]) -> HashSet<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn market_streams_request_snapshot() {
        assert!(wants_market_snapshot(&set(&["market_data"])));
        assert!(wants_market_snapshot(&set(&["all"])));
        assert!(wants_market_snapshot(&set(&["market.EURUSD"])));
    }

    #[test]
    fn non_market_streams_do_not_request_snapshot() {
        assert!(!wants_market_snapshot(&set(&["forex_news"])));
        assert!(!wants_market_snapshot(&set(&["x", "system"])));
    }

    #[test]
    fn snapshot_shapes_broadcast_envelope() {
        let shaped = shape_snapshot(json!({ "items": [], "total": 0 }));

        assert_eq!(shaped["event"], "market.snapshot");
        assert_eq!(shaped["channel"], "market_data");
        assert_eq!(shaped["data"]["total"], 0);
        assert!(shaped["timestamp"].is_string());
    }
}
