use futures_util::StreamExt;
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::hub::Hub;

#[derive(Debug, Deserialize)]
struct BroadcastEnvelope {
    event: String,
    data: Value,
    channel: String,
}

pub async fn run(redis_url: String, prefix: String, hub: Arc<Hub>) {
    loop {
        if let Err(err) = subscribe_loop(&redis_url, &prefix, &hub).await {
            error!(error = %err, "realtime Redis subscriber failed, reconnecting in 5s");
        }
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}

async fn subscribe_loop(redis_url: &str, prefix: &str, hub: &Arc<Hub>) -> anyhow::Result<()> {
    let client = redis::Client::open(redis_url)?;
    let mut pubsub = client.get_async_pubsub().await?;
    let pattern = format!("{prefix}:*");
    pubsub.psubscribe(&pattern).await?;
    info!(pattern = %pattern, "realtime gateway subscribed to Redis fanout");

    while let Some(message) = pubsub.on_message().next().await {
        let payload: String = message.get_payload()?;
        match serde_json::from_str::<BroadcastEnvelope>(&payload) {
            Ok(envelope) => {
                hub.broadcast(&envelope.event, envelope.data, &envelope.channel)
                    .await;
            }
            Err(err) => {
                warn!(error = %err, "failed to parse realtime Redis envelope");
            }
        }
    }

    Ok(())
}
