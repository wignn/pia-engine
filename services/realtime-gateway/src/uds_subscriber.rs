use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn};

use crate::hub::Hub;

pub async fn run(socket_path: String, hub: Arc<Hub>) {
    loop {
        info!(path = %socket_path, "connecting to ingestion UDS hot-path socket...");
        match atlsd_common::ipc::UdsReceiver::connect(&socket_path, Duration::from_secs(3)).await {
            Ok(mut receiver) => {
                info!("connected to ingestion UDS hot-path! Receiving sub-50µs live ticks");
                while let Ok(bytes) = receiver.recv().await {
                    if let Ok(text) = std::str::from_utf8(&bytes) {
                        if let Ok(Some(tick)) = parse_uds_tick(text) {
                            hub.broadcast("market.trade", json!({ "tick": tick }), "market_data")
                                .await;
                        }
                    }
                }
                warn!("UDS hot-path socket stream ended, reconnecting in 2s...");
            }
            Err(err) => {
                warn!(
                    error = %err,
                    path = %socket_path,
                    "UDS socket connect failed (ingestion not ready); retrying in 3s"
                );
            }
        }
        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}

fn parse_uds_tick(payload: &str) -> anyhow::Result<Option<Value>> {
    let mut tick: Value = serde_json::from_str(payload)?;
    let Some(object) = tick.as_object_mut() else {
        return Ok(None);
    };

    let price = object
        .get("price")
        .and_then(|value| value.as_f64())
        .unwrap_or(0.0);
    if price <= 0.0 {
        return Ok(None);
    }

    object.insert("source".to_string(), json!("market_data"));
    Ok(Some(tick))
}
