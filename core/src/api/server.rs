use std::collections::HashSet;
use std::sync::Arc;

use axum::{
    extract::{State, WebSocketUpgrade},
    middleware,
    response::Response,
    routing::{get, post},
    Router,
};
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing::info;

use crate::api::handlers;
use crate::api::middleware::{optional_api_key_auth, strict_api_key_auth, usage_logger};
use crate::api::state::AppState;
use crate::ws;

pub fn build_router(state: AppState) -> Router {
    let allowed_origins: Vec<axum::http::HeaderValue> = [
        "http://localhost:3000",
        "http://localhost:5173",
        "http://localhost:8080",
        "https://forex.wign.cloud",
        "http://forex.wign.cloud",
    ]
    .iter()
    .filter_map(|o| o.parse().ok())
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
        // --- General WebSocket (for Bot / all channels) ---
        .route("/api/v1/ws", get(ws_general_handler))
        // --- WebSocket streams — each route auto-subscribes to its channel ---
        .route("/api/v1/ws/market", get(ws_market_handler))
        .route("/api/v1/ws/market/{symbol}", get(ws_handler_single_symbol))
        .route("/api/v1/ws/news", get(ws_news_handler))
        .route("/api/v1/ws/equity", get(ws_equity_handler))
        .route("/api/v1/ws/calendar", get(ws_calendar_handler))
        .route("/api/v1/ws/x", get(ws_x_handler))
        .route("/api/v1/ws/x/{symbol}", get(ws_handler_single_symbol))
        // --- Market prices (REST) ---
        .route("/api/v1/market/prices", get(handlers::market::list_prices))
        .route("/api/v1/market/prices/{symbol}", get(handlers::market::get_price))
        // --- Forex ---
        .route("/api/v1/forex/news", get(handlers::news::list_news))
        .route("/api/v1/forex/news/latest", get(handlers::news::latest_news))
        .route("/api/v1/forex/news/{id}", get(handlers::news::get_news))
        .route("/api/v1/forex/calendar", get(handlers::calendar::list_calendar))
        // --- Equity / Stock ---
        .route("/api/v1/equity/news", get(handlers::stock::latest_stock_news))
        .layer(middleware::from_fn_with_state(state.clone(), usage_logger))
        .layer(middleware::from_fn_with_state(state.clone(), optional_api_key_auth));

    let private_api = Router::new()
        .route("/api/v1/content/scrape", post(handlers::scraping::scrape_article))
        .layer(middleware::from_fn_with_state(state.clone(), usage_logger))
        .layer(middleware::from_fn_with_state(state.clone(), strict_api_key_auth));

    Router::new()
        .merge(public_api)
        .merge(private_api)
        .layer(cors)
        .with_state(state)
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

    let token = params.get("token").or_else(|| params.get("api_key")).cloned();

    let mut user_id = None;
    let mut tv_symbols = HashSet::new();

    let channels_query: Option<HashSet<String>> = channel_override
        .map(|ch| std::iter::once(ch.to_string()).collect())
        .or_else(|| {
            params
                .get("channels")
                .map(|c| c.split(',').map(|s| s.trim().to_string()).collect())
        });

    let symbols_query = params
        .get("symbols")
        .map(|s| s.split(',').map(|s| s.trim().to_string()).collect::<HashSet<_>>())
        .unwrap_or_default();

    if let Some(raw_key) = &token {
        if state.config.api_keys.contains(raw_key) {
            tv_symbols = symbols_query;
        } else if let Some(registry) = &state.tenant_registry {
            if let Some(ctx) = registry.validate_key(raw_key).await {
                let current = state.hub.user_connection_count(&ctx.user_id).await;
                if current >= ctx.ws_connections as usize {
                    return axum::response::Response::builder()
                        .status(429)
                        .body(axum::body::Body::from("WebSocket connection limit reached for your plan"))
                        .unwrap();
                }
                user_id = Some(ctx.user_id);
                if !symbols_query.is_empty() {
                    tv_symbols = ctx.tv_symbols.intersection(&symbols_query).cloned().collect();
                } else {
                    tv_symbols = ctx.tv_symbols;
                }
            }
        }
    }

    let hub = state.hub.clone();
    ws.on_upgrade(move |socket| {
        ws::client::handle_socket(socket, hub, bot_id, user_id, HashSet::new(), tv_symbols, channels_query)
    })
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

async fn ws_news_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Response {
    ws_handler_inner(ws, state, params, Some("news")).await
}

async fn ws_equity_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Response {
    ws_handler_inner(ws, state, params, Some("equity_news")).await
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
    axum::extract::Query(mut params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Response {
    params.insert("symbols".to_string(), symbol);
    ws_handler_inner(ws, state, params, Some("market_data")).await
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

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
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
