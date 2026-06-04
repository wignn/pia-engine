use axum::{routing::get, Router};
use tower_http::cors::{Any, CorsLayer};

use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/health", get(crate::health))
        .route("/api/v1/forex/calendar", get(crate::news::list_calendar))
        .route("/api/v1/forex/news", get(crate::news::list_forex_news))
        .route(
            "/api/v1/forex/news/latest",
            get(crate::news::latest_forex_news),
        )
        .route("/api/v1/forex/news/{id}", get(crate::news::get_forex_news))
        .route(
            "/api/v1/forex/sources/status",
            get(crate::news::source_statuses),
        )
        .route("/api/v1/stock/news", get(crate::news::latest_stock_news))
        .route("/api/v1/macro/dashboard", get(crate::news::macro_dashboard))
        .layer(cors)
        .with_state(state)
}
