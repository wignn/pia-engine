use std::collections::{hash_map::DefaultHasher, HashSet};
use std::hash::{Hash, Hasher};

use axum::{
    extract::{Path, State, WebSocketUpgrade},
    http::StatusCode,
    response::Response,
    routing::{get, post},
    Json, Router,
};
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

use crate::state::{AppState, Ticket};

pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/health", get(crate::health))
        .route("/ws/v1", get(ws_v1_handler))
        .route("/api/v1/ws/v1", get(ws_v1_handler))
        .route("/api/v1/ws", get(ws_general_handler))
        .route("/api/v1/ws/market", get(ws_market_handler))
        .route("/api/v1/ws/market/{symbol}", get(ws_handler_single_symbol))
        .route("/api/v1/ws/forex-news", get(ws_forex_news_handler))
        .route("/api/v1/ws/stock", get(ws_stock_handler))
        .route("/api/v1/ws/calendar", get(ws_calendar_handler))
        .route("/api/v1/ws/x", get(ws_x_handler))
        .route("/api/v1/ws/x/{username}", get(ws_handler_single_x_username))
        .route("/api/v1/ws/ticket", post(generate_ws_ticket))
        .layer(cors)
        .with_state(state)
}

fn normalize_symbol_set(raw: &str) -> HashSet<String> {
    raw.split(',')
        .map(|symbol| symbol.trim().to_uppercase())
        .filter(|symbol| !symbol.is_empty())
        .collect()
}

fn text_response(status: StatusCode, body: &'static str) -> Response {
    axum::response::Response::builder()
        .status(status)
        .body(axum::body::Body::from(body))
        .unwrap()
}

fn string_response(status: StatusCode, body: String) -> Response {
    axum::response::Response::builder()
        .status(status)
        .body(axum::body::Body::from(body))
        .unwrap()
}

fn legacy_streams(
    channels: &Option<HashSet<String>>,
    symbols: &HashSet<String>,
) -> Result<HashSet<String>, crate::streams::StreamError> {
    let channels = channels
        .clone()
        .unwrap_or_else(crate::client::default_channels);
    let mut streams = HashSet::new();

    for channel in channels {
        if channel.is_empty() || channel == "__empty__" {
            continue;
        }
        if channel == "market_data" {
            if symbols.is_empty() {
                streams.insert("market_data".to_string());
            } else {
                for symbol in symbols {
                    streams.insert(crate::streams::parse_stream(&format!(
                        "market_data:{symbol}"
                    ))?);
                }
            }
        } else {
            streams.insert(crate::streams::parse_stream(&channel)?);
        }
    }

    Ok(streams)
}

async fn ws_handler_inner(
    ws: WebSocketUpgrade,
    state: AppState,
    params: std::collections::HashMap<String, String>,
    channel_override: Option<&str>,
) -> Response {
    let bot_id = params
        .get("bot_id")
        .or_else(|| params.get("client_type"))
        .cloned()
        .unwrap_or_else(|| "unknown".into());

    let mut token = params
        .get("token")
        .or_else(|| params.get("api_key"))
        .cloned();

    if let Some(ticket_id) = params.get("ticket") {
        let mut store = state.ticket_store.write().await;
        if let Some(ticket) = store.remove(ticket_id) {
            if std::time::Instant::now() < ticket.expires_at {
                token = Some(ticket.api_key);
            }
        }
    }

    let Some(raw_key) = token.as_ref() else {
        return text_response(
            StatusCode::UNAUTHORIZED,
            "Valid API key required for WebSocket connection",
        );
    };
    let tenant_context = match &state.tenant_registry {
        Some(registry) => registry.validate_key(raw_key).await,
        None => None,
    };
    let authenticated = tenant_context.is_some() || state.config.api_keys.contains(raw_key);
    if !authenticated {
        return text_response(
            StatusCode::UNAUTHORIZED,
            "Valid API key required for WebSocket connection",
        );
    }
    let api_key_id = tenant_context
        .as_ref()
        .map(|tenant| tenant.api_key_id.to_string())
        .unwrap_or_else(|| {
            let mut hasher = DefaultHasher::new();
            raw_key.hash(&mut hasher);
            hasher.finish().to_string()
        });
    let connection_limit = tenant_context
        .as_ref()
        .map(|tenant| tenant.ws_connections)
        .or_else(|| state.config.api_key_connection_limits.get(raw_key).copied())
        .unwrap_or(i32::MAX);
    if !state
        .hub
        .try_acquire_api_key_slot(&api_key_id, connection_limit)
        .await
    {
        return text_response(
            StatusCode::TOO_MANY_REQUESTS,
            "WebSocket connection limit reached",
        );
    }

    let channels_query: Option<HashSet<String>> = channel_override
        .map(|ch| std::iter::once(ch.to_string()).collect())
        .or_else(|| {
            params
                .get("channels")
                .map(|channels| channels.split(',').map(|s| s.trim().to_string()).collect())
        });

    let symbols_query = params
        .get("symbols")
        .map(|symbols| normalize_symbol_set(symbols))
        .unwrap_or_default();

    let initial_streams = match legacy_streams(&channels_query, &symbols_query) {
        Ok(streams) => streams,
        Err(error) => return string_response(StatusCode::BAD_REQUEST, error.message),
    };

    let user_id: Option<Uuid> = tenant_context.as_ref().map(|tenant| tenant.user_id);
    let hub = state.hub.clone();
    ws.on_upgrade(move |socket| async move {
        let (client_id, rx) = hub
            .register_api_key(bot_id.clone(), initial_streams, user_id, api_key_id)
            .await;
        crate::client::handle_registered_socket(socket, hub, client_id, rx, tenant_context).await;
    })
}

#[derive(serde::Serialize)]
struct TicketResponse {
    ticket: String,
}

async fn generate_ws_ticket(
    State(state): State<AppState>,
    request: axum::extract::Request,
) -> Result<Json<TicketResponse>, StatusCode> {
    let api_key = request
        .headers()
        .get("X-API-Key")
        .or_else(|| request.headers().get(axum::http::header::AUTHORIZATION))
        .and_then(|h| h.to_str().ok())
        .map(|s| s.strip_prefix("Bearer ").unwrap_or(s).to_string())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let valid = match &state.tenant_registry {
        Some(registry) => registry.validate_key(&api_key).await.is_some(),
        None => false,
    } || state.config.api_keys.contains(&api_key);
    if !valid {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let ticket_id = Uuid::new_v4().to_string();
    let ticket = Ticket {
        api_key,
        expires_at: std::time::Instant::now() + std::time::Duration::from_secs(30),
    };

    state
        .ticket_store
        .write()
        .await
        .insert(ticket_id.clone(), ticket);
    let mut store = state.ticket_store.write().await;
    let now = std::time::Instant::now();
    store.retain(|_, ticket| ticket.expires_at > now);

    Ok(Json(TicketResponse { ticket: ticket_id }))
}

async fn ws_v1_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Response {
    let mut params = params;
    params.insert("channels".to_string(), "".to_string());
    ws_handler_inner(ws, state, params, Some("__empty__")).await
}

async fn ws_general_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Response {
    ws_handler_inner(ws, state, params, None).await
}

async fn ws_market_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Response {
    ws_handler_inner(ws, state, params, Some("market_data")).await
}

async fn ws_forex_news_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Response {
    ws_handler_inner(ws, state, params, Some("forex_news")).await
}

async fn ws_stock_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Response {
    ws_handler_inner(ws, state, params, Some("stock_news")).await
}

async fn ws_calendar_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Response {
    ws_handler_inner(ws, state, params, Some("calendar")).await
}

async fn ws_x_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Response {
    ws_handler_inner(ws, state, params, Some("x")).await
}

async fn ws_handler_single_symbol(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(symbol): Path<String>,
    axum::extract::Query(mut params): axum::extract::Query<
        std::collections::HashMap<String, String>,
    >,
) -> Response {
    params.insert("symbols".to_string(), symbol.to_uppercase());
    ws_handler_inner(ws, state, params, Some("market_data")).await
}

async fn ws_handler_single_x_username(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(username): Path<String>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Response {
    let channel = format!("x:{username}");
    ws_handler_inner(ws, state, params, Some(&channel)).await
}
