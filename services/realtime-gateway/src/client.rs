use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

use atlsd_domain::tenant::TenantContext;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::streams;

#[derive(Debug, Deserialize)]
struct ClientCommand {
    #[serde(alias = "action")]
    method: String,
    #[serde(default, alias = "symbols", alias = "streams")]
    params: Vec<String>,
    #[serde(default)]
    id: Option<Value>,
    #[serde(default)]
    api_key: Option<String>,
    #[serde(default)]
    token: Option<String>,
    #[serde(default)]
    ticket: Option<String>,
}

#[allow(dead_code)]
pub struct ClientHandle {
    pub id: crate::hub::ClientId,
    pub bot_id: String,
    pub user_id: Option<Uuid>,
    pub api_key_id: Option<String>,
    pub streams: HashSet<String>,
    pub sender: mpsc::Sender<Arc<str>>,
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

pub async fn handle_registered_socket(
    socket: axum::extract::ws::WebSocket,
    hub: Arc<crate::hub::Hub>,
    client_id: crate::hub::ClientId,
    mut rx: mpsc::Receiver<Arc<str>>,
    tenant_context: Option<TenantContext>,
    snapshot: Arc<crate::snapshot::Snapshot>,
) {
    use axum::extract::ws::Message;
    use futures_util::{SinkExt, StreamExt};
    use std::time::Duration;
    use tracing::{debug, warn};

    let (control_tx, mut control_rx) = mpsc::channel::<Arc<str>>(64);
    let (mut ws_tx, mut ws_rx) = socket.split();

    let write_hub = hub.clone();
    let write_task = tokio::spawn(async move {
        let mut ping_interval = tokio::time::interval(Duration::from_secs(
            crate::hub::Hub::connection_counter_refresh_sec(),
        ));

        loop {
            tokio::select! {
                Some(msg) = rx.recv() => {
                    write_hub.metrics().message_out();
                    if ws_tx.send(Message::Text(msg.as_ref().into())).await.is_err() {
                        break;
                    }
                }
                Some(msg) = control_rx.recv() => {
                    write_hub.metrics().message_out();
                    if ws_tx.send(Message::Text(msg.as_ref().into())).await.is_err() {
                        break;
                    }
                }
                _ = ping_interval.tick() => {
                    if let Some(api_key_id) = write_hub.client_api_key_id(client_id).await {
                        write_hub.refresh_api_key_slot(&api_key_id).await;
                    }
                    write_hub.metrics().ping();
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

    let read_hub = hub.clone();
    let read_task = tokio::spawn(async move {
        let timeout = Duration::from_secs(120);
        loop {
            match tokio::time::timeout(timeout, ws_rx.next()).await {
                Ok(Some(Ok(Message::Pong(_)))) => {
                    read_hub.metrics().pong();
                    debug!(client_id, "pong received");
                }
                Ok(Some(Ok(Message::Text(text)))) => {
                    read_hub.metrics().message_in();
                    handle_command(
                        client_id,
                        &read_hub,
                        &control_tx,
                        tenant_context.as_ref(),
                        text.as_str(),
                        &snapshot,
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

    tokio::select! {
        _ = write_task => {}
        _ = read_task => {}
    }

    hub.unregister(client_id).await;
}

pub async fn handle_unauthenticated_socket(
    mut socket: axum::extract::ws::WebSocket,
    state: crate::state::AppState,
    bot_id: String,
    channel_override: Option<String>,
    params: std::collections::HashMap<String, String>,
) {
    use axum::extract::ws::Message;
    use futures_util::SinkExt;
    use std::time::Duration;

    let welcome = json!({
        "event": "connected",
        "data": {
            "message": "Connected to PIA Realtime Gateway. Authentication required within 5 seconds.",
            "format": {"action": "auth", "api_key": "YOUR_KEY"}
        }
    });
    if socket
        .send(Message::Text(welcome.to_string().into()))
        .await
        .is_err()
    {
        return;
    }

    let auth_timeout = Duration::from_secs(5);
    let msg = match tokio::time::timeout(auth_timeout, socket.recv()).await {
        Ok(Some(Ok(Message::Text(text)))) => text,
        _ => {
            let timeout_err = json!({
                "error": "auth_timeout",
                "message": "Authentication timed out. Disconnecting."
            });
            let _ = socket
                .send(Message::Text(timeout_err.to_string().into()))
                .await;
            let _ = socket.close().await;
            return;
        }
    };

    let cmd = match serde_json::from_str::<ClientCommand>(&msg) {
        Ok(cmd) => cmd,
        Err(_) => {
            let parse_err = json!({
                "error": "bad_request",
                "message": "Expected JSON payload with {\"action\": \"auth\", \"api_key\": \"...\"}"
            });
            let _ = socket
                .send(Message::Text(parse_err.to_string().into()))
                .await;
            let _ = socket.close().await;
            return;
        }
    };

    if cmd.method.to_uppercase() != "AUTH" {
        let auth_req_err = json!({
            "error": "unauthenticated",
            "message": "First message must be an auth action: {\"action\": \"auth\", \"api_key\": \"...\"}"
        });
        let _ = socket
            .send(Message::Text(auth_req_err.to_string().into()))
            .await;
        let _ = socket.close().await;
        return;
    }

    let mut token = cmd.api_key.or(cmd.token);
    if let Some(ticket_id) = cmd.ticket {
        if let Some(api_key) = state.ticket_store.redeem(&ticket_id).await {
            token = Some(api_key);
        }
    }

    let Some(raw_key) = token else {
        let missing_key_err = json!({
            "error": "unauthorized",
            "message": "Missing 'api_key' or 'ticket' in auth frame"
        });
        let _ = socket
            .send(Message::Text(missing_key_err.to_string().into()))
            .await;
        let _ = socket.close().await;
        return;
    };

    let (api_key_id, tenant_context, admin_authenticated) =
        match crate::http::authenticate_and_verify(&state, &raw_key).await {
            Ok(tuple) => tuple,
            Err((status, msg)) => {
                let err_msg = json!({
                    "error": status.as_u16(),
                    "message": msg
                });
                let _ = socket.send(Message::Text(err_msg.to_string().into())).await;
                let _ = socket.close().await;
                return;
            }
        };

    let connection_limit = if admin_authenticated {
        i32::MAX
    } else {
        tenant_context
            .as_ref()
            .map(|tenant| tenant.ws_connections)
            .or_else(|| {
                state
                    .config
                    .api_key_connection_limits
                    .get(&raw_key)
                    .copied()
            })
            .unwrap_or(i32::MAX)
    };

    if !state
        .hub
        .try_acquire_api_key_slot(&api_key_id, connection_limit)
        .await
    {
        let limit_err = json!({
            "error": "rate_limited",
            "message": "WebSocket connection limit reached"
        });
        let _ = socket
            .send(Message::Text(limit_err.to_string().into()))
            .await;
        let _ = socket.close().await;
        return;
    }

    let ack = json!({
        "event": "authenticated",
        "id": cmd.id.unwrap_or(Value::Null),
        "data": {
            "user_id": tenant_context.as_ref().map(|t| t.user_id),
            "plan": tenant_context.as_ref().map(|t| t.plan.as_str()).unwrap_or("enterprise"),
            "ws_connections_max": connection_limit
        }
    });
    if socket
        .send(Message::Text(ack.to_string().into()))
        .await
        .is_err()
    {
        return;
    }

    let channels_query: Option<HashSet<String>> = channel_override
        .map(|ch| std::iter::once(ch).collect())
        .or_else(|| {
            params
                .get("channels")
                .map(|channels| channels.split(',').map(|s| s.trim().to_string()).collect())
        });

    let symbols_query = params
        .get("symbols")
        .map(|symbols| crate::http::normalize_symbol_set(symbols))
        .unwrap_or_default();

    let initial_streams =
        crate::http::legacy_streams(&channels_query, &symbols_query).unwrap_or_default();

    let user_id: Option<Uuid> = tenant_context.as_ref().map(|tenant| tenant.user_id);
    let wants_snapshot = crate::snapshot::wants_market_snapshot(&initial_streams);
    let (client_id, rx) = state
        .hub
        .register_api_key(bot_id, initial_streams, user_id, api_key_id)
        .await;

    if wants_snapshot {
        let snapshot = state.snapshot.clone();
        let hub = state.hub.clone();
        tokio::spawn(async move {
            crate::snapshot::send_snapshot(&snapshot, &hub, client_id).await;
        });
    }

    handle_registered_socket(
        socket,
        state.hub.clone(),
        client_id,
        rx,
        tenant_context,
        state.snapshot.clone(),
    )
    .await;
}

async fn handle_command(
    client_id: crate::hub::ClientId,
    hub: &Arc<crate::hub::Hub>,
    control_tx: &mpsc::Sender<Arc<str>>,
    tenant_context: Option<&TenantContext>,
    text: &str,
    snapshot: &Arc<crate::snapshot::Snapshot>,
) {
    hub.metrics().command();
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
        "AUTH" => {
            send_control(
                control_tx,
                json!({
                    "event": "authenticated",
                    "status": "already_authenticated",
                    "id": command.id.unwrap_or(Value::Null),
                }),
            )
            .await;
        }
        "SUBSCRIBE" => {
            let normalized_params: Vec<String> = command
                .params
                .into_iter()
                .map(|p| {
                    let s = p.trim();
                    if !s.contains(':')
                        && !crate::streams::BASE_STREAMS.contains(&s.to_lowercase().as_str())
                    {
                        format!("market_data:{}", s.to_uppercase())
                    } else {
                        s.to_string()
                    }
                })
                .collect();
            let streams = match streams::normalize_streams(&normalized_params) {
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
            hub.subscribe(client_id, streams.clone()).await;
            // Catch-up: a fresh market subscription first receives the current
            // latest-price snapshot, then continues with the live stream.
            if crate::snapshot::wants_market_snapshot(&streams) {
                let snapshot = snapshot.clone();
                let hub = hub.clone();
                tokio::spawn(async move {
                    crate::snapshot::send_snapshot(&snapshot, &hub, client_id).await;
                });
            }
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

async fn send_control(control_tx: &mpsc::Sender<Arc<str>>, value: Value) {
    if let Ok(payload) = serde_json::to_string(&value) {
        let _ = control_tx.send(Arc::from(payload)).await;
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

    #[test]
    fn client_command_parses_action_auth() {
        let json_auth = r#"{"action": "auth", "api_key": "wi_live_test123"}"#;
        let cmd: ClientCommand = serde_json::from_str(json_auth).unwrap();
        assert_eq!(cmd.method.to_uppercase(), "AUTH");
        assert_eq!(cmd.api_key.as_deref(), Some("wi_live_test123"));
    }

    #[test]
    fn client_command_parses_symbols_alias() {
        let json_sub = r#"{"action": "subscribe", "symbols": ["XAUUSD", "BTCUSDT"]}"#;
        let cmd: ClientCommand = serde_json::from_str(json_sub).unwrap();
        assert_eq!(cmd.method.to_uppercase(), "SUBSCRIBE");
        assert_eq!(cmd.params, vec!["XAUUSD", "BTCUSDT"]);
    }

    #[test]
    fn client_command_parses_token_and_ticket() {
        let json_ticket = r#"{"action": "auth", "ticket": "wst_12345"}"#;
        let cmd: ClientCommand = serde_json::from_str(json_ticket).unwrap();
        assert_eq!(cmd.ticket.as_deref(), Some("wst_12345"));

        let json_token = r#"{"method": "auth", "token": "wi_live_tok"}"#;
        let cmd2: ClientCommand = serde_json::from_str(json_token).unwrap();
        assert_eq!(cmd2.token.as_deref(), Some("wi_live_tok"));
    }
}
