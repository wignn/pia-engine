use std::collections::HashSet;

use axum::{
    extract::{State, WebSocketUpgrade},
    http::StatusCode,
    middleware,
    response::Response,
    routing::{get, post},
    Json, Router,
};
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing::info;

use crate::api::handlers;
use crate::api::middleware::{optional_api_key_auth, strict_api_key_auth, usage_logger};
use crate::api::state::AppState;
use crate::ws;

pub fn build_router(state: AppState) -> Router {
    let allowed_origins: Vec<axum::http::HeaderValue> = std::env::var("ALLOWED_ORIGINS")
        .unwrap_or_else(|_| {
            "http://localhost:3000,http://localhost:5173,http://localhost:8080,https://forex.wign.cloud,http://forex.wign.cloud,https://fio-page.vercel.app,https://fio.wign.dev".to_string()
        })
        .split(',')
        .filter_map(|o| o.trim().parse().ok())
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(allowed_origins))
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::HeaderName::from_static("x-api-key"),
            axum::http::header::AUTHORIZATION,
        ])
        .allow_credentials(true);

    let public_api = Router::new()
        .route("/health", get(handlers::health::health))
        .route("/", get(handlers::health::root))
        .route("/api/v1/market/prices", get(handlers::market::list_prices))
        .route(
            "/api/v1/market/prices/{symbol}",
            get(handlers::market::get_price),
        )
        .route(
            "/api/v1/market/history/{symbol}",
            get(handlers::market::get_history),
        )
        .route("/api/v1/forex/news", get(handlers::forex::list_forex_news))
        .route(
            "/api/v1/forex/news/latest",
            get(handlers::forex::latest_forex_news),
        )
        .route(
            "/api/v1/forex/news/{id}",
            get(handlers::forex::get_forex_news),
        )
        .route(
            "/api/v1/forex/calendar",
            get(handlers::calendar::list_calendar),
        )
        .route(
            "/api/v1/stock/news",
            get(handlers::stock::latest_stock_news),
        )
        .route("/api/v1/analyze", post(handlers::sentiment::analyze_text))
        .layer(middleware::from_fn_with_state(state.clone(), usage_logger))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            optional_api_key_auth,
        ));

    let ws_api = Router::new()
        .route("/api/v1/ws", get(ws_general_handler))
        .route("/api/v1/ws/market", get(ws_market_handler))
        .route("/api/v1/ws/market/{symbol}", get(ws_handler_single_symbol))
        .route("/api/v1/ws/forex-news", get(ws_forex_news_handler))
        .route("/api/v1/ws/stock", get(ws_stock_handler))
        .route("/api/v1/ws/calendar", get(ws_calendar_handler))
        .route("/api/v1/ws/x", get(ws_x_handler))
        .route("/api/v1/ws/x/{username}", get(ws_handler_single_x_username))
        .layer(middleware::from_fn_with_state(state.clone(), usage_logger))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            strict_api_key_auth,
        ));

    let ticket_api = Router::new()
        .route("/api/v1/ws/ticket", post(generate_ws_ticket))
        .layer(middleware::from_fn_with_state(state.clone(), usage_logger))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            strict_api_key_auth,
        ));

    let private_api = Router::new()
        .route(
            "/api/v1/content/scrape",
            post(handlers::scraping::scrape_article),
        )
        .layer(middleware::from_fn_with_state(state.clone(), usage_logger))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            strict_api_key_auth,
        ));

    Router::new()
        .merge(public_api)
        .merge(ws_api)
        .merge(ticket_api)
        .merge(private_api)
        .layer(cors)
        .with_state(state)
}

fn is_market_subscription(channels: &Option<HashSet<String>>) -> bool {
    channels
        .as_ref()
        .map(|channels| channels.contains("all") || channels.contains("market_data"))
        .unwrap_or(true)
}

fn normalize_symbol_set(raw: &str) -> HashSet<String> {
    raw.split(',')
        .map(|symbol| symbol.trim().to_uppercase())
        .filter(|symbol| !symbol.is_empty())
        .collect()
}

fn resolve_market_symbols(
    allowed: &HashSet<String>,
    requested: &HashSet<String>,
) -> Result<HashSet<String>, &'static str> {
    if allowed.is_empty() {
        return Err("No market symbols configured for your plan");
    }

    if requested.is_empty() {
        return Ok(allowed.clone());
    }

    if requested.is_subset(allowed) {
        return Ok(requested.clone());
    }

    Err("Requested market symbol is not allowed by your plan")
}

fn text_response(status: StatusCode, body: &'static str) -> Response {
    axum::response::Response::builder()
        .status(status)
        .body(axum::body::Body::from(body))
        .unwrap()
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

    let mut user_id = None;
    let mut tv_symbols = HashSet::new();
    let mut authenticated = false;

    let channels_query: Option<HashSet<String>> = channel_override
        .map(|ch| std::iter::once(ch.to_string()).collect())
        .or_else(|| {
            params
                .get("channels")
                .map(|c| c.split(',').map(|s| s.trim().to_string()).collect())
        });

    let symbols_query = params
        .get("symbols")
        .map(|symbols| normalize_symbol_set(symbols))
        .unwrap_or_default();

    if let Some(raw_key) = &token {
        if state.config.api_keys.contains(raw_key) {
            tv_symbols = symbols_query;
            authenticated = true;
        } else if let Some(registry) = &state.tenant_registry {
            if let Some(ctx) = registry.validate_key(raw_key).await {
                let current = state.hub.user_connection_count(&ctx.user_id).await;
                if current >= ctx.ws_connections as usize {
                    return axum::response::Response::builder()
                        .status(429)
                        .body(axum::body::Body::from(
                            "WebSocket connection limit reached for your plan",
                        ))
                        .unwrap();
                }
                user_id = Some(ctx.user_id);
                authenticated = true;

                if is_market_subscription(&channels_query) {
                    match resolve_market_symbols(&ctx.tv_symbols, &symbols_query) {
                        Ok(symbols) => tv_symbols = symbols,
                        Err(message) => return text_response(StatusCode::FORBIDDEN, message),
                    }
                }
            }
        }
    }

    if !authenticated {
        return text_response(
            StatusCode::UNAUTHORIZED,
            "Valid API key required for WebSocket connection",
        );
    }

    let hub = state.hub.clone();
    ws.on_upgrade(move |socket| {
        ws::client::handle_socket(
            socket,
            hub,
            bot_id,
            user_id,
            HashSet::new(),
            tv_symbols,
            channels_query,
        )
    })
}

#[derive(serde::Serialize)]
struct TicketResponse {
    ticket: String,
}

async fn generate_ws_ticket(
    State(state): State<AppState>,
    request: axum::extract::Request,
) -> Result<Json<TicketResponse>, axum::http::StatusCode> {
    let api_key = request
        .headers()
        .get("X-API-Key")
        .or_else(|| request.headers().get(axum::http::header::AUTHORIZATION))
        .and_then(|h| h.to_str().ok())
        .map(|s| s.strip_prefix("Bearer ").unwrap_or(s).to_string())
        .ok_or(axum::http::StatusCode::UNAUTHORIZED)?;

    let ticket_id = uuid::Uuid::new_v4().to_string();

    let ticket = crate::api::state::Ticket {
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
    store.retain(|_, t| t.expires_at > now);

    Ok(Json(TicketResponse { ticket: ticket_id }))
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
    axum::extract::Path(symbol): axum::extract::Path<String>,
    axum::extract::Query(mut params): axum::extract::Query<
        std::collections::HashMap<String, String>,
    >,
) -> Response {
    params.insert("symbols".to_string(), symbol);
    ws_handler_inner(ws, state, params, Some("market_data")).await
}

async fn ws_handler_single_x_username(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    axum::extract::Path(username): axum::extract::Path<String>,
    axum::extract::Query(mut params): axum::extract::Query<
        std::collections::HashMap<String, String>,
    >,
) -> Response {
    params.insert("x_username".to_string(), username);
    ws_handler_inner(ws, state, params, Some("x")).await
}

pub async fn start(state: AppState) -> Result<(), Box<dyn std::error::Error>> {
    let port = state.config.server_port;
    let app = build_router(state);
    let addr = format!("0.0.0.0:{}", port);

    info!(addr = %addr, "HTTP server starting");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn symbols(values: &[&str]) -> HashSet<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn normalize_symbol_set_trims_uppercases_and_dedupes() {
        let normalized = normalize_symbol_set(" eurusd, GBPUSD ,, eurusd ");

        assert_eq!(normalized, symbols(&["EURUSD", "GBPUSD"]));
    }

    #[test]
    fn market_symbols_default_to_allowed_symbols() {
        let allowed = symbols(&["EURUSD", "GBPUSD"]);
        let requested = HashSet::new();

        assert_eq!(
            resolve_market_symbols(&allowed, &requested).unwrap(),
            allowed
        );
    }

    #[test]
    fn market_symbols_accept_allowed_subset() {
        let allowed = symbols(&["EURUSD", "GBPUSD"]);
        let requested = symbols(&["EURUSD"]);

        assert_eq!(
            resolve_market_symbols(&allowed, &requested).unwrap(),
            requested
        );
    }

    #[test]
    fn market_symbols_reject_empty_allowlist() {
        let allowed = HashSet::new();
        let requested = HashSet::new();

        assert!(resolve_market_symbols(&allowed, &requested).is_err());
    }

    #[test]
    fn market_symbols_reject_unallowed_requested_symbol() {
        let allowed = symbols(&["EURUSD", "GBPUSD"]);
        let requested = symbols(&["EURUSD", "XAUUSD"]);

        assert!(resolve_market_symbols(&allowed, &requested).is_err());
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("received Ctrl+C, shutting down"),
        _ = terminate => info!("received SIGTERM, shutting down"),
    }
}
