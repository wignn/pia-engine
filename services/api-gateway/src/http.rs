use crate::state::AppState;
use axum::{
    middleware,
    routing::{any, get},
    Json, Router,
};
use serde_json::{json, Value};
use tower_http::cors::{Any, CorsLayer};

pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let public = Router::new()
        .route("/health", get(health))
        .route("/", get(root));

    let protected = Router::new()
        .route("/api/v1/market/prices", any(crate::proxy::proxy_request))
        .route(
            "/api/v1/market/prices/{symbol}",
            any(crate::proxy::proxy_request),
        )
        .route(
            "/api/v1/market/history/{symbol}",
            any(crate::proxy::proxy_request),
        )
        .route(
            "/api/v1/market/session/{symbol}",
            any(crate::proxy::proxy_request),
        )
        .route(
            "/api/v1/market/data-quality",
            any(crate::proxy::proxy_request),
        )
        .route("/api/v1/market/spikes", any(crate::proxy::proxy_request))
        .route("/api/v1/market/alerts", any(crate::proxy::proxy_request))
        .route(
            "/api/v1/market/smart-alerts",
            any(crate::proxy::proxy_request),
        )
        .route(
            "/api/v1/market/economic/indicators",
            any(crate::proxy::proxy_request),
        )
        .route(
            "/api/v1/market/economic/indicators/{series_id}",
            any(crate::proxy::proxy_request),
        )
        .route(
            "/api/v1/market/economic/latest",
            any(crate::proxy::proxy_request),
        )
        .route(
            "/api/v1/market/economic/countries",
            any(crate::proxy::proxy_request),
        )
        .route(
            "/api/v1/market/economic/categories",
            any(crate::proxy::proxy_request),
        )
        .route(
            "/api/v1/market/why/{symbol}",
            any(crate::proxy::proxy_request),
        )
        .route("/api/v1/analyze", any(crate::proxy::proxy_request))
        .route("/api/v1/forex/calendar", any(crate::proxy::proxy_request))
        .route("/api/v1/forex/news", any(crate::proxy::proxy_request))
        .route(
            "/api/v1/forex/news/latest",
            any(crate::proxy::proxy_request),
        )
        .route("/api/v1/forex/news/{id}", any(crate::proxy::proxy_request))
        .route(
            "/api/v1/forex/sources/status",
            any(crate::proxy::proxy_request),
        )
        .route("/api/v1/stock/news", any(crate::proxy::proxy_request))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            crate::auth::usage_logger,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            crate::auth::require_api_key_auth,
        ));

    Router::new()
        .merge(public)
        .merge(protected)
        .layer(cors)
        .with_state(state)
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "healthy", "service": "api-gateway" }))
}

async fn root() -> Json<Value> {
    Json(json!({ "service": "ATLSD API Gateway", "version": "1.0.0" }))
}
