use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::error;

use crate::state::AppState;

#[derive(Deserialize)]
pub struct GeosignalsQuery {
    pub limit: Option<i64>,
    pub country: Option<String>,
    pub region: Option<String>,
    pub category: Option<String>,
    pub min_severity: Option<f64>,
}

/// Normalizes the limit query parameter with defaults and clamping.
/// Default: 50, Range: 1..200
fn normalized_limit(limit: Option<i64>) -> i64 {
    limit.unwrap_or(50).clamp(1, 200)
}

/// Normalizes the min_severity query parameter with defaults and clamping.
/// Default: 0.0, Range: 0.0..1.0
fn normalized_min_severity(min_severity: Option<f64>) -> f64 {
    min_severity.unwrap_or(0.0).clamp(0.0, 1.0)
}

pub async fn list_geosignals(
    State(state): State<AppState>,
    Query(query): Query<GeosignalsQuery>,
) -> Json<Value> {
    let limit = normalized_limit(query.limit);
    let country = query.country.unwrap_or_default();
    let region = query.region.unwrap_or_default();
    let category = query.category.unwrap_or_default();
    let min_severity = normalized_min_severity(query.min_severity);

    let rows = sqlx::query_as::<_, (String, chrono::DateTime<chrono::Utc>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, f64, f64, f64, Option<Vec<String>>, serde_json::Value, Option<chrono::DateTime<chrono::Utc>>)>(
        "SELECT event_id, timestamp, source, source_url, title, summary, category, country, region, location_scope, severity_score, sentiment_score, confidence_score, affected_assets, asset_impact, freshness FROM news.geosignals WHERE ($2 = '' OR country ILIKE '%' || $2 || '%') AND ($3 = '' OR region ILIKE '%' || $3 || '%') AND ($4 = '' OR category ILIKE '%' || $4 || '%') AND severity_score >= $5 ORDER BY timestamp DESC LIMIT $1",
    )
    .bind(limit)
    .bind(country)
    .bind(region)
    .bind(category)
    .bind(min_severity)
    .fetch_all(&state.db)
    .await;

    match rows {
        Ok(rows) => {
            let items: Vec<Value> = rows
                .into_iter()
                .map(|row| {
                    json!({
                        "event_id": row.0,
                        "timestamp": row.1,
                        "source": row.2,
                        "source_url": row.3,
                        "title": row.4,
                        "summary": row.5,
                        "category": row.6,
                        "country": row.7,
                        "region": row.8,
                        "location_scope": row.9,
                        "severity_score": row.10,
                        "sentiment_score": row.11,
                        "confidence_score": row.12,
                        "affected_assets": row.13,
                        "asset_impact": row.14,
                        "freshness": row.15,
                    })
                })
                .collect();
            Json(json!({
                "items": items,
                "total": items.len(),
                "limit": limit,
            }))
        }
        Err(err) => {
            error!(error = %err, "geosignals query failed");
            Json(json!({ "error": "query failed" }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalized_limit_default() {
        assert_eq!(normalized_limit(None), 50);
    }

    #[test]
    fn test_normalized_limit_clamp_min() {
        assert_eq!(normalized_limit(Some(0)), 1);
        assert_eq!(normalized_limit(Some(-10)), 1);
    }

    #[test]
    fn test_normalized_limit_clamp_max() {
        assert_eq!(normalized_limit(Some(300)), 200);
        assert_eq!(normalized_limit(Some(1000)), 200);
    }

    #[test]
    fn test_normalized_limit_within_range() {
        assert_eq!(normalized_limit(Some(50)), 50);
        assert_eq!(normalized_limit(Some(100)), 100);
        assert_eq!(normalized_limit(Some(1)), 1);
        assert_eq!(normalized_limit(Some(200)), 200);
    }

    #[test]
    fn test_normalized_min_severity_default() {
        assert_eq!(normalized_min_severity(None), 0.0);
    }

    #[test]
    fn test_normalized_min_severity_clamp_min() {
        assert_eq!(normalized_min_severity(Some(-0.5)), 0.0);
        assert_eq!(normalized_min_severity(Some(-10.0)), 0.0);
    }

    #[test]
    fn test_normalized_min_severity_clamp_max() {
        assert_eq!(normalized_min_severity(Some(1.5)), 1.0);
        assert_eq!(normalized_min_severity(Some(10.0)), 1.0);
    }

    #[test]
    fn test_normalized_min_severity_within_range() {
        assert_eq!(normalized_min_severity(Some(0.0)), 0.0);
        assert_eq!(normalized_min_severity(Some(0.5)), 0.5);
        assert_eq!(normalized_min_severity(Some(1.0)), 1.0);
    }
}
