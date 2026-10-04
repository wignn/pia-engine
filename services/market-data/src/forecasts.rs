use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::FromRow;

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ForecastQuery {
    pub timeframe: Option<String>,
}

#[derive(Debug, FromRow)]
struct ForecastRow {
    symbol: String,
    timeframe: String,
    decision_at: DateTime<Utc>,
    horizon_end: DateTime<Utc>,
    reference_price: f64,
    down_probability: f64,
    flat_probability: f64,
    up_probability: f64,
    expected_return: f64,
    q10_return: f64,
    q50_return: f64,
    q90_return: f64,
    uncertainty: f64,
    model_version: String,
    data_version: String,
    feature_version: String,
    source_masks: Value,
    source_age_seconds: Value,
}

pub async fn latest_forecast(
    Path(symbol): Path<String>,
    Query(query): Query<ForecastQuery>,
    State(state): State<AppState>,
) -> (StatusCode, Json<Value>) {
    let symbol = symbol.to_ascii_uppercase();
    let timeframe = query.timeframe.unwrap_or_else(|| "15m".to_string());
    if symbol != "XAUUSD" || timeframe != "15m" {
        return (
            StatusCode::BAD_REQUEST,
            Json(
                json!({"error":"unsupported_forecast_scope","supported_symbol":"XAUUSD","supported_timeframe":"15m"}),
            ),
        );
    }

    let row = sqlx::query_as::<_, ForecastRow>(
        r#"SELECT symbol, timeframe, decision_at, horizon_end, reference_price,
                  down_probability, flat_probability, up_probability, expected_return,
                  q10_return, q50_return, q90_return, uncertainty, model_version,
                  data_version, feature_version, source_masks, source_age_seconds
           FROM market.market_forecasts
           WHERE symbol = $1 AND timeframe = $2
           ORDER BY decision_at DESC, created_at DESC, forecast_id DESC
           LIMIT 1"#,
    )
    .bind(&symbol)
    .bind(&timeframe)
    .fetch_optional(&state.db)
    .await;

    match row {
        Ok(Some(forecast)) => {
            let status = if Utc::now() < forecast.horizon_end {
                "active"
            } else {
                "expired"
            };
            (
                StatusCode::OK,
                Json(json!({
                    "status": status,
                    "symbol": forecast.symbol,
                    "timeframe": forecast.timeframe,
                    "decision_at": forecast.decision_at,
                    "horizon_end": forecast.horizon_end,
                    "reference_price": forecast.reference_price,
                    "probabilities": {
                        "down": forecast.down_probability,
                        "flat": forecast.flat_probability,
                        "up": forecast.up_probability
                    },
                    "expected_return": forecast.expected_return,
                    "return_quantiles": {
                        "q10": forecast.q10_return,
                        "q50": forecast.q50_return,
                        "q90": forecast.q90_return
                    },
                    "uncertainty": forecast.uncertainty,
                    "model_version": forecast.model_version,
                    "data_version": forecast.data_version,
                    "feature_version": forecast.feature_version,
                    "source_masks": forecast.source_masks,
                    "source_age_seconds": forecast.source_age_seconds,
                    "paper_only": true
                })),
            )
        }
        Ok(None) => (
            StatusCode::OK,
            Json(
                json!({"status":"unavailable","symbol":symbol,"timeframe":timeframe,"forecast":null}),
            ),
        ),
        Err(error) => {
            tracing::warn!(error = %error, "failed to load latest XAUUSD forecast");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({"error":"forecast_unavailable","retryable":true})),
            )
        }
    }
}
