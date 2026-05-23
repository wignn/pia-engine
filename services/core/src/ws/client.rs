use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

use atlsd_domain::tenant::TenantContext;
use serde::Deserialize;
use serde_json::{json, Value};

use super::streams;

#[derive(Debug, Deserialize)]
struct ClientCommand {
    method: String,
    #[serde(default)]
    params: Vec<String>,
    #[serde(default)]
    id: Option<Value>,
}

#[allow(dead_code)]
pub struct ClientHandle {
    pub id: super::hub::ClientId,
    pub bot_id: String,
    pub user_id: Option<Uuid>,
    pub streams: HashSet<String>,
    pub sender: mpsc::Sender<Vec<u8>>,
}

pub fn default_channels() -> HashSet<String> {
    [
        "all",
        "forex_news",
        "stock_news",
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

pub async fn handle_socket(
    socket: axum::extract::ws::WebSocket,
    hub: Arc<super::hub::Hub>,
    bot_id: String,
    user_id: Option<Uuid>,
    tenant_context: Option<TenantContext>,
    initial_streams: HashSet<String>,
) {
    use axum::extract::ws::Message;
    use futures_util::{SinkExt, StreamExt};
    use std::time::Duration;
    use tracing::{debug, warn};

    let (client_id, mut rx) = hub.register(bot_id.clone(), initial_streams, user_id).await;

    let (control_tx, mut control_rx) = mpsc::channel::<Vec<u8>>(64);
    let (mut ws_tx, mut ws_rx) = socket.split();

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
                Some(msg) = control_rx.recv() => {
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

    // Read pump: consume incoming messages and handle subscription commands.
    let read_hub = hub.clone();
    let read_task = tokio::spawn(async move {
        let timeout = Duration::from_secs(120);
        loop {
            match tokio::time::timeout(timeout, ws_rx.next()).await {
                Ok(Some(Ok(Message::Pong(_)))) => {
                    debug!(client_id, "pong received");
                }
                Ok(Some(Ok(Message::Text(text)))) => {
                    handle_command(
                        client_id,
                        &read_hub,
                        &control_tx,
                        tenant_context.as_ref(),
                        text.as_str(),
                    )
                    .await;
                }
                Ok(Some(Ok(Message::Close(_)))) | Ok(None) | Err(_) => break,
                Ok(Some(Err(e))) => {
                    warn!(client_id, error = %e, "ws read error");
                    break;
                }
                Ok(Some(Ok(_))) => {}
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

async fn handle_command(
    client_id: super::hub::ClientId,
    hub: &Arc<super::hub::Hub>,
    control_tx: &mpsc::Sender<Vec<u8>>,
    tenant_context: Option<&TenantContext>,
    text: &str,
) {
    let command = match serde_json::from_str::<ClientCommand>(text) {
        Ok(command) => command,
        Err(_) => {
            let error = streams::StreamError::bad_request("Invalid JSON command");
            send_control(control_tx, streams::error_response(&error, None)).await;
            return;
        }
    };

    let method = command.method.to_uppercase();
    match method.as_str() {
        "SUBSCRIBE" => {
            let streams = match streams::normalize_streams(&command.params) {
                Ok(streams) => streams,
                Err(error) => {
                    send_control(control_tx, streams::error_response(&error, command.id)).await;
                    return;
                }
            };
            let current = hub.list_subscriptions(client_id).await.unwrap_or_default();
            let current: HashSet<String> = current.into_iter().collect();
            if let Err(error) =
                streams::validate_subscription_change(tenant_context, &current, &streams)
            {
                send_control(control_tx, streams::error_response(&error, command.id)).await;
                return;
            }
            hub.subscribe(client_id, streams).await;
            send_control(
                control_tx,
                json!({ "result": Value::Null, "id": command.id.unwrap_or(Value::Null) }),
            )
            .await;
        }
        "UNSUBSCRIBE" => {
            let streams = match streams::normalize_streams(&command.params) {
                Ok(streams) => streams,
                Err(error) => {
                    send_control(control_tx, streams::error_response(&error, command.id)).await;
                    return;
                }
            };
            hub.unsubscribe(client_id, &streams).await;
            send_control(
                control_tx,
                json!({ "result": Value::Null, "id": command.id.unwrap_or(Value::Null) }),
            )
            .await;
        }
        "LIST_SUBSCRIPTIONS" => {
            let subscriptions = hub.list_subscriptions(client_id).await.unwrap_or_default();
            send_control(
                control_tx,
                json!({ "result": subscriptions, "id": command.id.unwrap_or(Value::Null) }),
            )
            .await;
        }
        "PING" => {
            send_control(
                control_tx,
                json!({ "result": "pong", "id": command.id.unwrap_or(Value::Null) }),
            )
            .await;
        }
        _ => {
            let error =
                streams::StreamError::bad_request(format!("Unknown method: {}", command.method));
            send_control(control_tx, streams::error_response(&error, command.id)).await;
        }
    }
}

async fn send_control(control_tx: &mpsc::Sender<Vec<u8>>, value: Value) {
    if let Ok(payload) = serde_json::to_vec(&value) {
        let _ = control_tx.send(payload).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_channels_include_expected_realtime_feeds() {
        let channels = default_channels();

        for channel in [
            "all",
            "forex_news",
            "stock_news",
            "calendar",
            "market_data",
            "x",
            "system",
        ] {
            assert!(channels.contains(channel));
        }
        assert_eq!(channels.len(), 9);
    }
}
