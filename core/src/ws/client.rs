use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

/// Handle stored per connected client for sending messages.
pub struct ClientHandle {
    pub id: super::hub::ClientId,
    pub bot_id: String,
    pub user_id: Option<Uuid>,
    pub channels: HashSet<String>,
    pub x_usernames: HashSet<String>,
    pub tv_symbols: HashSet<String>,
    pub sender: mpsc::Sender<Vec<u8>>,
}

/// Default channels a new WebSocket client subscribes to.
pub fn default_channels() -> HashSet<String> {
    [
        "all",
        "news",
        "equity_news",
        "high_impact",
        "calendar",
        "market_data",
        "volatility",
        "x",
        "system",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

/// Run the read/write pump for a connected WebSocket client.
pub async fn handle_socket(
    socket: axum::extract::ws::WebSocket,
    hub: Arc<super::hub::Hub>,
    bot_id: String,
    user_id: Option<Uuid>,
    x_usernames: HashSet<String>,
    tv_symbols: HashSet<String>,
) {
    use axum::extract::ws::Message;
    use futures_util::{SinkExt, StreamExt};
    use std::time::Duration;
    use tracing::{debug, warn};

    let channels = default_channels();
    let (client_id, mut rx) = hub.register(bot_id.clone(), channels, user_id, x_usernames, tv_symbols).await;

    let (mut ws_tx, mut ws_rx) = socket.split();

    // Write pump: forward messages from hub → WebSocket
    let write_hub = hub.clone();
    let write_task = tokio::spawn(async move {
        let mut ping_interval = tokio::time::interval(Duration::from_secs(30));

        loop {
            tokio::select! {
                Some(msg) = rx.recv() => {
                    let text = String::from_utf8_lossy(&msg).into_owned();
                    if ws_tx.send(Message::Text(text.into())).await.is_err() {
                        break;
                    }
                }
                _ = ping_interval.tick() => {
                    if ws_tx.send(Message::Ping(vec![].into())).await.is_err() {
                        break;
                    }
                }
                else => break,
            }
        }

        let _ = ws_tx.close().await;
        write_hub.unregister(client_id).await;
    });

    // Read pump: consume incoming messages (keep connection alive)
    let read_task = tokio::spawn(async move {
        let timeout = Duration::from_secs(120);
        loop {
            match tokio::time::timeout(timeout, ws_rx.next()).await {
                Ok(Some(Ok(Message::Pong(_)))) => {
                    debug!(client_id, "pong received");
                }
                Ok(Some(Ok(Message::Close(_)))) | Ok(None) | Err(_) => break,
                Ok(Some(Err(e))) => {
                    warn!(client_id, error = %e, "ws read error");
                    break;
                }
                Ok(Some(Ok(_))) => {} // ignore other messages
            }
        }
    });

    // Wait for either task to finish, then abort the other
    tokio::select! {
        _ = write_task => {}
        _ = read_task => {}
    }

    hub.unregister(client_id).await;
}
