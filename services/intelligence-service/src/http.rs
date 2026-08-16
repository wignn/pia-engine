use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};

use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    let internal_auth = atlsd_auth::internal::InternalAuth::from_env();

    Router::new()
        .route("/health", get(crate::health))
        .route("/api/v1/analyze", post(crate::sentiment::analyze_text))
        .route(
            "/api/v1/market/why/{symbol}",
            get(crate::why_move::why_did_it_move),
        )
        .layer(axum::middleware::from_fn_with_state(
            internal_auth,
            atlsd_auth::internal::require_internal_key,
        ))
        .layer(cors)
        .with_state(state)
}
