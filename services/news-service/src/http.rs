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
        .route(
            "/api/v1/admin/forex/sources",
            get(crate::news::admin_list_forex_sources).post(crate::news::admin_create_forex_source),
        )
        .route(
            "/api/v1/admin/forex/sources/test",
            post(crate::news::admin_test_forex_source),
        )
        .route(
            "/api/v1/admin/forex/sources/{id}",
            post(crate::news::admin_update_forex_source),
        )
        .route(
            "/api/v1/admin/forex/sources/{id}/toggle",
            post(crate::news::admin_toggle_forex_source),
        )
        .route("/api/v1/stock/news", get(crate::news::latest_stock_news))
        .route("/api/v1/macro/dashboard", get(crate::news::macro_dashboard))
        .route("/api/v1/geosignals/map", get(crate::geosignals::map_layers))
        .route(
            "/api/v1/geosignals/assets",
            get(crate::geosignals::asset_impacts),
        )
        .route(
            "/api/v1/geosignals/status",
            get(crate::geosignals::geosignal_status),
        )
        .route(
            "/api/v1/geosignals",
            get(crate::geosignals::list_geosignals),
        )
        .route("/api/v1/sec/filings", get(crate::sec::list_filings))
        .route(
            "/api/v1/sec/filings/{accession_number}",
            get(crate::sec::get_filing),
        )
        .route(
            "/api/v1/sec/companies/{symbol}",
            get(crate::sec::get_company),
        )
        .route(
            "/api/v1/central-banks/latest",
            get(crate::central_bank::list_latest_documents),
        )
        .route(
            "/api/v1/central-banks/{bank}/documents",
            get(crate::central_bank::list_bank_documents),
        )
        .route(
            "/api/v1/central-banks/{bank}/stance",
            get(crate::central_bank::get_bank_stance),
        )
        .layer(cors)
        .with_state(state)
}
