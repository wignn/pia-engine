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
        .route("/api/v1/market/prices", get(crate::prices::list_prices))
        .route(
            "/api/v1/market/prices/{symbol}",
            get(crate::prices::get_price),
        )
        .route(
            "/api/v1/market/history/{symbol}",
            get(crate::history::get_history),
        )
        .route(
            "/api/v1/market/session/{symbol}",
            get(crate::session::get_session),
        )
        .route(
            "/api/v1/market/data-quality",
            get(crate::data_quality::data_quality),
        )
        .route("/api/v1/market/spikes", get(crate::spikes::spikes))
        .route("/api/v1/market/alerts", get(crate::alerts::alerts))
        .route("/api/v1/market/smart-alerts", get(crate::alerts::alerts))
        .route(
            "/api/v1/market/economic/indicators",
            get(crate::economic::list_indicators),
        )
        .route(
            "/api/v1/market/economic/indicators/{series_id}",
            get(crate::economic::get_series),
        )
        .route(
            "/api/v1/market/economic/latest",
            get(crate::economic::latest_indicators),
        )
        .route(
            "/api/v1/market/economic/countries",
            get(crate::economic::list_countries),
        )
        .route(
            "/api/v1/market/economic/categories",
            get(crate::economic::list_categories),
        )
        .layer(cors)
        .with_state(state)
}
