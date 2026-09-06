use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub resolution: Option<String>,
    pub limit: Option<usize>,
    pub before: Option<i64>,
}

pub async fn get_history(
    Path(symbol): Path<String>,
    Query(query): Query<HistoryQuery>,
    State(state): State<AppState>,
) -> Json<Value> {
    let symbol = symbol.to_uppercase();
    let resolution = normalize_resolution(query.resolution.as_deref().unwrap_or("1m"));
    let limit = query.limit.unwrap_or(120).clamp(1, 1000);

    let Some(clickhouse) = &state.clickhouse else {
        tracing::error!(symbol = %symbol, "ClickHouse is required for market history");
        return Json(json!([]));
    };

    match clickhouse
        .latest_history(&symbol, &resolution, limit, query.before)
        .await
    {
        Ok(history) => Json(json!(history)),
        Err(err) => {
            tracing::warn!(error = %err, symbol = %symbol, "failed to load ClickHouse history");
            Json(json!([]))
        }
    }
}

pub fn normalize_resolution(raw: &str) -> String {
    match raw.trim().to_lowercase().as_str() {
        "1" | "1m" | "m1" => "1m".to_string(),
        "5" | "5m" | "m5" => "5m".to_string(),
        "15" | "15m" | "m15" => "15m".to_string(),
        "60" | "1h" | "h1" => "1h".to_string(),
        _ => "1m".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_supported_resolutions() {
        assert_eq!(normalize_resolution("M1"), "1m");
        assert_eq!(normalize_resolution("5"), "5m");
        assert_eq!(normalize_resolution("h1"), "1h");
        assert_eq!(normalize_resolution("bad"), "1m");
    }
}
