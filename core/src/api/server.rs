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

/// Build the Axum router with all routes and middleware.
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
        .route("/api/v1/ws/forex", get(ws_handler))
        .route("/api/v1/ws/equity", get(ws_handler))
        .route("/api/v1/ws/x", get(ws_handler))
        .route("/api/v1/ws/stats", get(stats_ws_handler))
        .route("/api/v1/forex/news", get(handlers::news::list_news))
        .route("/api/v1/forex/news/latest", get(handlers::news::latest_news))
        .route("/api/v1/forex/news/{id}", get(handlers::news::get_news))
        .route("/api/v1/equity/news/latest", get(handlers::stock::latest_stock_news))
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

/// Tenant-aware WebSocket handler.
/// Validates API key from query param, loads tenant config, enforces WS connection limit.
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Response {
    let bot_id = params
        .get("bot_id")
        .or_else(|| params.get("client_type"))
        .cloned()
        .unwrap_or_else(|| "unknown".into());

    let token = params.get("token").or_else(|| params.get("api_key")).cloned();

    // Try to resolve tenant context from token
    let mut user_id = None;
    let mut tv_symbols = HashSet::new();

    if let Some(raw_key) = &token {
        // Check env admin keys first
        if state.config.api_keys.contains(raw_key) {
            // Admin: no filtering, get all
        } else if let Some(registry) = &state.tenant_registry {
            if let Some(ctx) = registry.validate_key(raw_key).await {
                // Enforce WS connection limit
                let current = state.hub.user_connection_count(&ctx.user_id).await;
                if current >= ctx.ws_connections as usize {
                    return axum::response::Response::builder()
                        .status(429)
                        .body(axum::body::Body::from("WebSocket connection limit reached for your plan"))
                        .unwrap();
                }
                user_id = Some(ctx.user_id);
                tv_symbols = ctx.tv_symbols;
            }
        }
    }

    let hub = state.hub.clone();
    ws.on_upgrade(move |socket| ws::client::handle_socket(socket, hub, bot_id, user_id, HashSet::new(), tv_symbols))
}

async fn stats_ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    let stats_hub = state.stats_hub.clone();
    ws.on_upgrade(move |socket| async move {
        stats_hub.handle_socket(socket).await;
    })
}

/// Start the HTTP server with graceful shutdown.
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
