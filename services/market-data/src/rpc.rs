use futures_util::StreamExt;
use std::time::Duration;
use tracing::{error, info, warn};

use crate::prices;
use crate::state::AppState;

pub async fn run_rpc_responder(nats_url: String, state: AppState) {
    loop {
        info!(nats_url = %nats_url, "connecting to NATS for market-data RPC responder...");
        match async_nats::connect(&nats_url).await {
            Ok(client) => {
                info!("market-data NATS RPC responder connected");
                let subject = atlsd_eventbus::subjects::RPC_MARKET_PRICES_V1;
                let queue = "market-data-rpc";

                match client
                    .queue_subscribe(subject.to_string(), queue.to_string())
                    .await
                {
                    Ok(mut subscriber) => {
                        info!(subject = %subject, queue = %queue, "subscribed to market-data RPC requests");
                        while let Some(msg) = subscriber.next().await {
                            if let Some(reply) = msg.reply {
                                let (bytes, _cache_hit) =
                                    prices::compute_prices_snapshot_bytes(&state).await;
                                if let Err(err) = client.publish(reply, bytes).await {
                                    warn!(error = %err, "failed to send NATS RPC reply");
                                }
                            }
                        }
                        warn!("market-data RPC subscription stream closed, reconnecting in 2s...");
                    }
                    Err(err) => {
                        error!(error = %err, "failed to subscribe to NATS RPC queue; retrying in 3s");
                    }
                }
            }
            Err(err) => {
                warn!(error = %err, "failed to connect to NATS for RPC responder; retrying in 3s");
            }
        }
        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}
