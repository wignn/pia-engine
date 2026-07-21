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

#[derive(Deserialize)]
pub struct GeoSignalMapQuery {
    pub limit: Option<i64>,
    pub window_hours: Option<i64>,
    pub scope: Option<String>,
}

#[derive(Deserialize)]
pub struct GeoSignalAssetQuery {
    pub limit: Option<i64>,
    pub window_hours: Option<i64>,
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

fn normalized_map_limit(limit: Option<i64>) -> i64 {
    limit.unwrap_or(100).clamp(1, 250)
}

fn normalized_window_hours(window_hours: Option<i64>) -> i64 {
    window_hours.unwrap_or(168).clamp(1, 24 * 30)
}

fn normalized_map_scope(scope: Option<String>) -> &'static str {
    match scope.as_deref().map(str::trim) {
        Some("region") => "region",
        Some("global") => "global",
        Some("country") | None => "country",
        _ => "country",
    }
}

fn normalized_asset_impact_limit(limit: Option<i64>) -> i64 {
    limit.unwrap_or(50).clamp(1, 100)
}

pub async fn asset_impacts(
    State(state): State<AppState>,
    Query(query): Query<GeoSignalAssetQuery>,
) -> Json<Value> {
    let limit = normalized_asset_impact_limit(query.limit);
    let window_hours = normalized_window_hours(query.window_hours);

    let rows = sqlx::query_as::<_, (String, i64, f64, f64, chrono::DateTime<chrono::Utc>)>(
        "SELECT asset, COUNT(*)::BIGINT AS signal_count, AVG(severity_score) AS avg_severity, MAX(severity_score) AS max_severity, MAX(timestamp) AS latest_timestamp
         FROM news.geosignals
         CROSS JOIN LATERAL unnest(affected_assets) AS asset
         WHERE timestamp >= NOW() - ($1 * INTERVAL '1 hour')
         GROUP BY asset
         ORDER BY max_severity DESC, signal_count DESC, asset ASC
         LIMIT $2",
    )
    .bind(window_hours)
    .bind(limit)
    .fetch_all(&state.db)
    .await;

    match rows {
        Ok(rows) => {
            let items: Vec<Value> = rows
                .into_iter()
                .map(|row| {
                    json!({
                        "asset": row.0,
                        "signal_count": row.1,
                        "avg_severity": row.2,
                        "max_severity": row.3,
                        "latest_timestamp": row.4,
                    })
                })
                .collect();
            Json(json!({
                "items": items,
                "total": items.len(),
                "limit": limit,
                "window_hours": window_hours,
            }))
        }
        Err(err) => {
            error!(error = %err, "geosignal asset impact query failed");
            Json(json!({ "error": "query failed" }))
        }
    }
}

pub async fn map_layers(
    State(state): State<AppState>,
    Query(query): Query<GeoSignalMapQuery>,
) -> Json<Value> {
    let limit = normalized_map_limit(query.limit);
    let window_hours = normalized_window_hours(query.window_hours);
    let scope = normalized_map_scope(query.scope);

    let query_sql = match scope {
        "region" => {
            "SELECT region AS layer_key, category, COUNT(*)::BIGINT AS signal_count, AVG(severity_score) AS avg_severity, MAX(severity_score) AS max_severity, MAX(timestamp) AS latest_timestamp
             FROM news.geosignals
             WHERE region IS NOT NULL AND timestamp >= NOW() - ($1 * INTERVAL '1 hour')
             GROUP BY region, category
             ORDER BY max_severity DESC, signal_count DESC, region ASC
             LIMIT $2"
        }
        "global" => {
            "SELECT 'global' AS layer_key, category, COUNT(*)::BIGINT AS signal_count, AVG(severity_score) AS avg_severity, MAX(severity_score) AS max_severity, MAX(timestamp) AS latest_timestamp
             FROM news.geosignals
             WHERE timestamp >= NOW() - ($1 * INTERVAL '1 hour')
             GROUP BY category
             ORDER BY max_severity DESC, signal_count DESC, category ASC
             LIMIT $2"
        }
        _ => {
            "SELECT country AS layer_key, category, COUNT(*)::BIGINT AS signal_count, AVG(severity_score) AS avg_severity, MAX(severity_score) AS max_severity, MAX(timestamp) AS latest_timestamp
             FROM news.geosignals
             WHERE country IS NOT NULL AND timestamp >= NOW() - ($1 * INTERVAL '1 hour')
             GROUP BY country, category
             ORDER BY max_severity DESC, signal_count DESC, country ASC
             LIMIT $2"
        }
    };

    let rows = sqlx::query_as::<
        _,
        (
            String,
            Option<String>,
            i64,
            f64,
            f64,
            chrono::DateTime<chrono::Utc>,
        ),
    >(query_sql)
    .bind(window_hours)
    .bind(limit)
    .fetch_all(&state.db)
    .await;

    match rows {
        Ok(rows) => {
            let items: Vec<Value> = rows
                .into_iter()
                .map(|row| {
                    json!({
                        "scope": scope,
                        "key": row.0,
                        "category": row.1,
                        "signal_count": row.2,
                        "avg_severity": row.3,
                        "max_severity": row.4,
                        "latest_timestamp": row.5,
                    })
                })
                .collect();
            Json(json!({
                "items": items,
                "total": items.len(),
                "limit": limit,
                "window_hours": window_hours,
                "scope": scope,
            }))
        }
        Err(err) => {
            error!(error = %err, "geosignal map layer query failed");
            Json(json!({ "error": "query failed" }))
        }
    }
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

    let rows = sqlx::query_as::<_, (String, chrono::DateTime<chrono::Utc>, String, Option<String>, String, Option<String>, String, Option<String>, Option<String>, String, f64, f64, f64, Vec<String>, serde_json::Value, String)>(
        "SELECT event_id, timestamp, source, source_url, title, summary, category, country, region, location_scope, severity_score, sentiment_score, confidence_score, affected_assets, asset_impact, freshness FROM news.geosignals WHERE ($1 = '' OR country ILIKE '%' || $1 || '%') AND ($2 = '' OR region ILIKE '%' || $2 || '%') AND ($3 = '' OR category ILIKE '%' || $3 || '%') AND severity_score >= $4 ORDER BY timestamp DESC LIMIT $5",
    )
    .bind(country)
    .bind(region)
    .bind(category)
    .bind(min_severity)
    .bind(limit)
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

pub async fn geosignal_status(State(state): State<AppState>) -> Json<Value> {
    let totals = sqlx::query_as::<_, (i64, Option<chrono::DateTime<chrono::Utc>>)>(
        "SELECT COUNT(*)::BIGINT, MAX(timestamp) FROM news.geosignals",
    )
    .fetch_one(&state.db)
    .await;

    let raw_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)::BIGINT FROM geosignal_raw_events WHERE source = 'gdelt'",
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    match totals {
        Ok((count, latest_timestamp)) => Json(json!({
            "status": "active",
            "source": "gdelt",
            "total_signals": count,
            "raw_events_count": raw_count,
            "latest_signal_timestamp": latest_timestamp,
        })),
        Err(err) => {
            error!(error = %err, "geosignal status query failed");
            Json(json!({
                "status": "error",
                "source": "gdelt",
                "error": "query failed"
            }))
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

    #[test]
    fn test_normalized_map_limit_defaults_and_clamps() {
        assert_eq!(normalized_map_limit(None), 100);
        assert_eq!(normalized_map_limit(Some(0)), 1);
        assert_eq!(normalized_map_limit(Some(500)), 250);
        assert_eq!(normalized_map_limit(Some(50)), 50);
    }

    #[test]
    fn test_normalized_window_hours_defaults_and_clamps() {
        assert_eq!(normalized_window_hours(None), 168);
        assert_eq!(normalized_window_hours(Some(0)), 1);
        assert_eq!(normalized_window_hours(Some(24 * 90)), 24 * 30);
        assert_eq!(normalized_window_hours(Some(24)), 24);
    }

    #[test]
    fn test_normalized_map_scope_defaults_and_allows_known_scopes() {
        assert_eq!(normalized_map_scope(None), "country");
        assert_eq!(normalized_map_scope(Some("country".to_string())), "country");
        assert_eq!(normalized_map_scope(Some("region".to_string())), "region");
        assert_eq!(normalized_map_scope(Some("global".to_string())), "global");
        assert_eq!(normalized_map_scope(Some("bad".to_string())), "country");
    }

    #[test]
    fn test_normalized_asset_impact_limit_defaults_and_clamps() {
        assert_eq!(normalized_asset_impact_limit(None), 50);
        assert_eq!(normalized_asset_impact_limit(Some(0)), 1);
        assert_eq!(normalized_asset_impact_limit(Some(200)), 100);
        assert_eq!(normalized_asset_impact_limit(Some(25)), 25);
    }
}
