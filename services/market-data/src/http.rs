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
            "/api/v1/market/trading-halts",
            get(crate::institutional::get_trading_halts),
        )
        .route(
            "/api/v1/market/corporate-actions",
            get(crate::institutional::get_corporate_actions),
        )
        .route(
            "/api/v1/market/realized-volatility",
            get(crate::institutional::get_realized_volatility),
        )
        .route(
            "/api/v1/market/implied-volatility",
            get(crate::institutional::get_implied_volatility),
        )
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
        .route(
            "/api/v1/rates/yield-curve",
            get(crate::rates::get_yield_curve),
        )
        .route("/api/v1/rates/spreads", get(crate::rates::get_spreads))
        .route(
            "/api/v1/rates/history/{tenor}",
            get(crate::rates::get_history),
        )
        .route("/api/v1/energy/series", get(crate::energy::list_series))
        .route(
            "/api/v1/energy/dashboard",
            get(crate::energy::energy_dashboard),
        )
        .route(
            "/api/v1/energy/{series_id}",
            get(crate::energy::get_series_observations),
        )
        .route("/api/v1/cot/markets", get(crate::cot::list_cot_markets))
        .route(
            "/api/v1/cot/symbol/{symbol}",
            get(crate::cot::get_cot_by_symbol),
        )
        .route(
            "/api/v1/cot/{market_code}",
            get(crate::cot::get_cot_by_market),
        )
        .route(
            "/api/v1/fear-greed",
            get(crate::fear_greed::get_fear_greed_latest),
        )
        .route(
            "/api/v1/fear-greed/history",
            get(crate::fear_greed::get_fear_greed_history),
        )
        .route(
            "/api/v1/fear-greed/components",
            get(crate::fear_greed::get_fear_greed_components),
        )
        .route(
            "/api/v1/options/summary",
            get(crate::options::get_options_summary),
        )
        .route(
            "/api/v1/options/chain",
            get(crate::options::get_options_chain),
        )
        .route("/api/v1/options/gex", get(crate::options::get_options_gex))
        .layer(cors)
        .with_state(state)
}
