use axum::extract::{Query, State};
use axum::Json;
use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;
use tracing::error;

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct BondYieldCurveQuery {
    pub country: Option<String>,
    pub window: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
struct BondSnapshotRow {
    country: String,
    as_of: NaiveDate,
    raw_json: serde_json::Value,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredBondDashboard {
    #[serde(default)]
    source: String,
    #[serde(default)]
    fetched_at: String,
    #[serde(default)]
    bonds: Vec<StoredBondQuote>,
    #[serde(default)]
    histories: HashMap<String, Vec<StoredHistoryPoint>>,
    #[serde(default)]
    history_available: bool,
    #[serde(default)]
    history_kind: String,
    #[serde(default)]
    history_message: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StoredBondQuote {
    pub symbol: String,
    pub name: String,
    #[serde(rename = "yield")]
    pub yield_value: f64,
    #[serde(rename = "dayChange")]
    pub day_change: f64,
    #[serde(rename = "monthChange")]
    pub month_change: f64,
    #[serde(rename = "yearChange")]
    pub year_change: f64,
    pub date: String,
}

#[derive(Debug, Deserialize)]
struct StoredHistoryPoint {
    date: String,
    value: f64,
}

#[derive(Debug, Serialize)]
pub struct BondYieldCurveResponse {
    pub country: String,
    pub source: String,
    pub as_of: NaiveDate,
    pub fetched_at: Option<String>,
    pub updated_at: DateTime<Utc>,
    pub window: String,
    pub window_from: NaiveDate,
    pub window_to: NaiveDate,
    pub stale: bool,
    pub history_available: bool,
    pub history_kind: String,
    pub history_message: Option<String>,
    pub bonds: Vec<StoredBondQuote>,
    pub history: Vec<BondHistorySeries>,
}

#[derive(Debug, Serialize)]
pub struct BondHistorySeries {
    pub symbol: String,
    pub name: String,
    pub points: Vec<BondHistoryPoint>,
}

#[derive(Debug, Serialize)]
pub struct BondHistoryPoint {
    pub date: NaiveDate,
    pub value: f64,
}

pub async fn get_yield_curve(
    Query(query): Query<BondYieldCurveQuery>,
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let country = query.country.as_deref().unwrap_or("US").trim();
    let country = if country.is_empty() { "US" } else { country };
    let window_days = parse_window_days(query.window.as_deref());
    let today = Utc::now().date_naive();
    let window_from = today - Duration::days(window_days - 1);
    let window_label = format!("{window_days}d");

    let row = sqlx::query_as::<_, BondSnapshotRow>(
        "SELECT country, as_of, raw_json, updated_at
         FROM macro_bonds
         WHERE country = $1
         ORDER BY as_of DESC, updated_at DESC
         LIMIT 1",
    )
    .bind(country)
    .fetch_optional(&state.db)
    .await;

    match row {
        Ok(Some(row)) => match build_response(row, window_from, today, window_label) {
            Ok(response) => Json(serde_json::json!(response)),
            Err(err) => {
                error!(error = %err, country, "failed to decode scraped bond snapshot");
                Json(serde_json::json!({ "error": "bond snapshot is invalid" }))
            }
        },
        Ok(None) => Json(serde_json::json!({
            "country": country,
            "source": "tradingeconomics",
            "window": window_label,
            "window_from": window_from,
            "window_to": today,
            "stale": true,
            "history_available": false,
            "history_kind": "unavailable",
            "history_message": "No scraped bond snapshot is available yet.",
            "bonds": [],
            "history": [],
        })),
        Err(err) => {
            error!(error = %err, country, "failed to query scraped bond snapshot");
            Json(serde_json::json!({ "error": "internal server error" }))
        }
    }
}

fn parse_window_days(window: Option<&str>) -> i64 {
    let Some(window) = window else { return 7 };
    let value = window.trim().strip_suffix('d').unwrap_or(window.trim());
    value
        .parse::<i64>()
        .ok()
        .filter(|days| (1..=31).contains(days))
        .unwrap_or(7)
}

fn build_response(
    row: BondSnapshotRow,
    window_from: NaiveDate,
    window_to: NaiveDate,
    window: String,
) -> Result<BondYieldCurveResponse, serde_json::Error> {
    let dashboard: StoredBondDashboard = serde_json::from_value(row.raw_json)?;
    let names: HashMap<&str, &str> = dashboard
        .bonds
        .iter()
        .map(|bond| (bond.symbol.as_str(), bond.name.as_str()))
        .collect();
    let provider_history = dashboard.history_kind == "provider" && dashboard.history_available;
    let mut history = Vec::new();

    if provider_history {
        for (symbol, points) in dashboard.histories {
            let mut filtered: Vec<BondHistoryPoint> = points
                .into_iter()
                .filter_map(|point| {
                    let date = parse_history_date(&point.date)?;
                    if date < window_from || date > window_to || !point.value.is_finite() {
                        return None;
                    }
                    Some(BondHistoryPoint {
                        date,
                        value: point.value,
                    })
                })
                .collect();
            filtered.sort_by_key(|point| point.date);
            filtered.dedup_by_key(|point| point.date);
            if !filtered.is_empty() {
                history.push(BondHistorySeries {
                    name: names
                        .get(symbol.as_str())
                        .copied()
                        .unwrap_or(symbol.as_str())
                        .to_string(),
                    symbol,
                    points: filtered,
                });
            }
        }
        history.sort_by(|left, right| left.symbol.cmp(&right.symbol));
    }

    let history_available = provider_history && !history.is_empty();
    let history_message = if history_available {
        None
    } else {
        Some(if dashboard.history_message.trim().is_empty() {
            "Authentic provider history is unavailable for this window.".to_string()
        } else {
            dashboard.history_message
        })
    };
    let fetched_at = (!dashboard.fetched_at.trim().is_empty()).then_some(dashboard.fetched_at);
    let stale = row.updated_at < Utc::now() - Duration::hours(24);

    Ok(BondYieldCurveResponse {
        country: row.country,
        source: if dashboard.source.trim().is_empty() {
            "tradingeconomics".to_string()
        } else {
            dashboard.source
        },
        as_of: row.as_of,
        fetched_at,
        updated_at: row.updated_at,
        window,
        window_from,
        window_to,
        stale,
        history_available,
        history_kind: if provider_history {
            "provider".to_string()
        } else {
            "unavailable".to_string()
        },
        history_message,
        bonds: dashboard.bonds,
        history,
    })
}

fn parse_history_date(value: &str) -> Option<NaiveDate> {
    let value = value.trim();
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .ok()
        .or_else(|| {
            DateTime::parse_from_rfc3339(value)
                .ok()
                .map(|date| date.date_naive())
        })
        .or_else(|| NaiveDate::parse_from_str(value, "%m/%d/%Y").ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_a_week() {
        assert_eq!(parse_window_days(None), 7);
        assert_eq!(parse_window_days(Some("7d")), 7);
        assert_eq!(parse_window_days(Some("365d")), 7);
    }

    #[test]
    fn parses_provider_dates() {
        assert_eq!(
            parse_history_date("2026-08-27"),
            NaiveDate::from_ymd_opt(2026, 8, 27)
        );
        assert_eq!(
            parse_history_date("2026-08-27T12:30:00Z"),
            NaiveDate::from_ymd_opt(2026, 8, 27)
        );
        assert_eq!(
            parse_history_date("8/27/2026"),
            NaiveDate::from_ymd_opt(2026, 8, 27)
        );
    }
}
