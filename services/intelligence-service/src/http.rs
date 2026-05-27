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

    Router::new()
        .route("/health", get(crate::health))
        .route("/api/v1/analyze", post(crate::sentiment::analyze_text))
        .route(
            "/api/v1/market/why/{symbol}",
            get(crate::why_move::why_did_it_move),
        )
        .layer(cors)
        .with_state(state)
}
