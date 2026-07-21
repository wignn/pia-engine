use axum::extract::{Path, Query, State};
use axum::Json;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::Duration;
use tracing::{error, info, warn};

use crate::config::Config;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RateObservation {
    pub source: String,
    pub country: String,
    pub tenor: String,
    pub date: NaiveDate,
    pub value: f64,
    pub unit: String,
    pub raw_series_id: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RateSpreadObservation {
    pub country: String,
    pub spread: String,
    pub date: NaiveDate,
    pub value: f64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct YieldCurveData {
    pub country: String,
    pub source: String,
    pub date: Option<NaiveDate>,
    pub points: Vec<RateObservation>,
    pub spreads: Vec<RateSpreadObservation>,
    pub stale: bool,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct YieldCurveQuery {
    pub country: Option<String>,
    pub date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct SpreadsQuery {
    pub country: Option<String>,
    pub spread: Option<String>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub country: Option<String>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub limit: Option<i64>,
}

#[cfg(test)]
fn calculate_spread(tenors: &[(String, f64)], short_tenor: &str, long_tenor: &str) -> Option<f64> {
    let short_val = tenors
        .iter()
        .find(|(t, _)| t == short_tenor)
        .map(|(_, v)| *v)?;
    let long_val = tenors
        .iter()
        .find(|(t, _)| t == long_tenor)
        .map(|(_, v)| *v)?;
    Some(long_val - short_val)
}

pub async fn get_yield_curve(
    State(state): State<AppState>,
    Query(params): Query<YieldCurveQuery>,
) -> Json<serde_json::Value> {
    let country = params.country.as_deref().unwrap_or("US");
    let result = query_yield_curve(&state.db, country, params.date).await;

    match result {
        Ok(yc) => Json(serde_json::json!(yc)),
        Err(err) => {
            error!(error = %err, country = %country, "failed to query yield curve");
            Json(serde_json::json!({ "error": "internal server error" }))
        }
    }
}

pub async fn get_spreads(
    State(state): State<AppState>,
    Query(params): Query<SpreadsQuery>,
) -> Json<serde_json::Value> {
    let country = params.country.as_deref().unwrap_or("US");
    let limit = params.limit.unwrap_or(100).clamp(1, 500);

    let result = query_spreads(
        &state.db,
        country,
        params.spread.as_deref(),
        params.from,
        params.to,
        limit,
    )
    .await;

    match result {
        Ok(data) => Json(serde_json::json!({ "data": data })),
        Err(err) => {
            error!(error = %err, country = %country, "failed to query spreads");
            Json(serde_json::json!({ "error": "internal server error" }))
        }
    }
}

pub async fn get_history(
    State(state): State<AppState>,
    Path(tenor): Path<String>,
    Query(params): Query<HistoryQuery>,
) -> Json<serde_json::Value> {
    let country = params.country.as_deref().unwrap_or("US");
    let limit = params.limit.unwrap_or(100).clamp(1, 500);

    let result = query_history(&state.db, country, &tenor, params.from, params.to, limit).await;

    match result {
        Ok(data) => Json(serde_json::json!({ "data": data })),
        Err(err) => {
            error!(error = %err, country = %country, tenor = %tenor, "failed to query rate history");
            Json(serde_json::json!({ "error": "internal server error" }))
        }
    }
}

async fn query_yield_curve(
    pool: &PgPool,
    country: &str,
    target_date: Option<NaiveDate>,
) -> Result<YieldCurveData, sqlx::Error> {
    let points = sqlx::query_as::<_, RateObservation>(
        r#"
        SELECT source, country, tenor, date, value, unit, raw_series_id, updated_at
        FROM (
            SELECT DISTINCT ON (tenor)
                source, country, tenor, date, value, unit, raw_series_id, updated_at
            FROM macro_rates
            WHERE country = $1 AND ($2::date IS NULL OR date <= $2)
            ORDER BY tenor, date DESC
        ) latest
        ORDER BY
          CASE tenor
            WHEN '3M' THEN 1
            WHEN '2Y' THEN 2
            WHEN '5Y' THEN 3
            WHEN '10Y' THEN 4
            WHEN '30Y' THEN 5
            ELSE 6
          END, tenor
        "#,
    )
    .bind(country)
    .bind(target_date)
    .fetch_all(pool)
    .await?;

    let spreads = query_latest_spreads(pool, country, target_date).await?;
    let date = points.iter().map(|p| p.date).max();
    let updated_at = points
        .iter()
        .map(|p| p.updated_at)
        .chain(spreads.iter().map(|s| s.updated_at))
        .max();
    let source = points
        .first()
        .map(|p| p.source.clone())
        .unwrap_or_else(|| "fred".to_string());
    let stale = points.iter().any(|p| Some(p.date) != target_date.or(date));

    Ok(YieldCurveData {
        country: country.to_string(),
        source,
        date,
        points,
        spreads,
        stale,
        updated_at,
    })
}

async fn query_spreads(
    pool: &PgPool,
    country: &str,
    spread: Option<&str>,
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
    limit: i64,
) -> Result<Vec<RateSpreadObservation>, sqlx::Error> {
    sqlx::query_as::<_, RateSpreadObservation>(
        r#"
        SELECT country, spread, date, value, updated_at
        FROM macro_rate_spreads
        WHERE country = $1
          AND ($2::text IS NULL OR spread = $2)
          AND ($3::date IS NULL OR date >= $3)
          AND ($4::date IS NULL OR date <= $4)
        ORDER BY date DESC
        LIMIT $5
        "#,
    )
    .bind(country)
    .bind(spread)
    .bind(from)
    .bind(to)
    .bind(limit)
    .fetch_all(pool)
    .await
}

async fn query_latest_spreads(
    pool: &PgPool,
    country: &str,
    target_date: Option<NaiveDate>,
) -> Result<Vec<RateSpreadObservation>, sqlx::Error> {
    sqlx::query_as::<_, RateSpreadObservation>(
        r#"
        SELECT country, spread, date, value, updated_at
        FROM (
            SELECT DISTINCT ON (spread) country, spread, date, value, updated_at
            FROM macro_rate_spreads
            WHERE country = $1 AND ($2::date IS NULL OR date <= $2)
            ORDER BY spread, date DESC
        ) latest
        ORDER BY spread
        "#,
    )
    .bind(country)
    .bind(target_date)
    .fetch_all(pool)
    .await
}

async fn query_history(
    pool: &PgPool,
    country: &str,
    tenor: &str,
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
    limit: i64,
) -> Result<Vec<RateObservation>, sqlx::Error> {
    sqlx::query_as::<_, RateObservation>(
        r#"
        SELECT source, country, tenor, date, value, unit, raw_series_id, updated_at
        FROM macro_rates
        WHERE country = $1
          AND tenor = $2
          AND ($3::date IS NULL OR date >= $3)
          AND ($4::date IS NULL OR date <= $4)
        ORDER BY date DESC
        LIMIT $5
        "#,
    )
    .bind(country)
    .bind(tenor)
    .bind(from)
    .bind(to)
    .bind(limit)
    .fetch_all(pool)
    .await
}

// --- Background FRED Sync for Rates ---

struct RateSeriesConfig {
    id: &'static str,
    country: &'static str,
    tenor: &'static str,
    unit: &'static str,
}

const RATE_SERIES_LIST: &[RateSeriesConfig] = &[
    RateSeriesConfig {
        id: "DGS3MO",
        country: "US",
        tenor: "3M",
        unit: "percent",
    },
    RateSeriesConfig {
        id: "DGS2",
        country: "US",
        tenor: "2Y",
        unit: "percent",
    },
    RateSeriesConfig {
        id: "DGS5",
        country: "US",
        tenor: "5Y",
        unit: "percent",
    },
    RateSeriesConfig {
        id: "DGS10",
        country: "US",
        tenor: "10Y",
        unit: "percent",
    },
    RateSeriesConfig {
        id: "DGS30",
        country: "US",
        tenor: "30Y",
        unit: "percent",
    },
    RateSeriesConfig {
        id: "DFII10",
        country: "US",
        tenor: "10Y_REAL",
        unit: "percent",
    },
    RateSeriesConfig {
        id: "T10YIE",
        country: "US",
        tenor: "10Y_BREAKEVEN",
        unit: "percent",
    },
];

struct SpreadSeriesConfig {
    id: &'static str,
    country: &'static str,
    spread: &'static str,
}

const SPREAD_SERIES_LIST: &[SpreadSeriesConfig] = &[
    SpreadSeriesConfig {
        id: "T10Y2Y",
        country: "US",
        spread: "10Y2Y",
    },
    SpreadSeriesConfig {
        id: "T10Y3M",
        country: "US",
        spread: "10Y3M",
    },
];

#[derive(Debug, Deserialize)]
struct FredObservationsResponse {
    observations: Vec<FredObservation>,
}

#[derive(Debug, Deserialize)]
struct FredObservation {
    date: String,
    value: String,
}

pub async fn run_rates_sync(config: Config, pool: PgPool) {
    if !config.has_fred() {
        warn!("FRED_API_KEY not set, rates data sync disabled");
        return;
    }

    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap_or_default();

    info!(
        refresh_sec = config.rates_refresh_sec,
        rate_series = RATE_SERIES_LIST.len(),
        spread_series = SPREAD_SERIES_LIST.len(),
        "rates data sync started"
    );

    loop {
        sync_all_rates(&config, &pool, &http).await;
        tokio::time::sleep(Duration::from_secs(config.rates_refresh_sec)).await;
    }
}

async fn sync_all_rates(config: &Config, pool: &PgPool, http: &reqwest::Client) {
    info!("starting rates FRED data sync");
    let mut success = 0u32;
    let mut failed = 0u32;

    for series in RATE_SERIES_LIST {
        match sync_rate_series(config, pool, http, series).await {
            Ok(count) => {
                success += 1;
                if count > 0 {
                    info!(
                        series_id = series.id,
                        observations = count,
                        "synced rate series"
                    );
                }
            }
            Err(err) => {
                failed += 1;
                error!(series_id = series.id, error = %err, "rate series sync failed");
            }
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    for series in SPREAD_SERIES_LIST {
        match sync_spread_series(config, pool, http, series).await {
            Ok(count) => {
                success += 1;
                if count > 0 {
                    info!(
                        series_id = series.id,
                        observations = count,
                        "synced spread series"
                    );
                }
            }
            Err(err) => {
                failed += 1;
                error!(series_id = series.id, error = %err, "spread series sync failed");
            }
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    info!(success, failed, "rates FRED sync complete");
}

async fn sync_rate_series(
    config: &Config,
    pool: &PgPool,
    http: &reqwest::Client,
    series: &RateSeriesConfig,
) -> anyhow::Result<usize> {
    let url = format!(
        "https://api.stlouisfed.org/fred/series/observations?series_id={}&api_key={}&file_type=json&sort_order=desc&limit=100",
        series.id, config.fred_api_key
    );

    let resp = http.get(&url).send().await?;
    if !resp.status().is_success() {
        anyhow::bail!("FRED API returned {}", resp.status());
    }

    let body: FredObservationsResponse = resp.json().await?;
    let mut count = 0usize;

    for obs in &body.observations {
        if obs.value == "." {
            continue;
        }
        let value: f64 = match obs.value.parse() {
            Ok(v) => v,
            Err(_) => continue,
        };
        let date = match NaiveDate::parse_from_str(&obs.date, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => continue,
        };

        sqlx::query(
            r#"
            INSERT INTO macro_rates (source, country, tenor, date, value, unit, raw_series_id, created_at, updated_at)
            VALUES ('fred', $1, $2, $3, $4, $5, $6, NOW(), NOW())
            ON CONFLICT (source, country, tenor, date) DO UPDATE SET
                value = EXCLUDED.value,
                unit = EXCLUDED.unit,
                raw_series_id = EXCLUDED.raw_series_id,
                updated_at = NOW()
            "#,
        )
        .bind(series.country)
        .bind(series.tenor)
        .bind(date)
        .bind(value)
        .bind(series.unit)
        .bind(series.id)
        .execute(pool)
        .await?;

        count += 1;
    }

    Ok(count)
}

async fn sync_spread_series(
    config: &Config,
    pool: &PgPool,
    http: &reqwest::Client,
    series: &SpreadSeriesConfig,
) -> anyhow::Result<usize> {
    let url = format!(
        "https://api.stlouisfed.org/fred/series/observations?series_id={}&api_key={}&file_type=json&sort_order=desc&limit=100",
        series.id, config.fred_api_key
    );

    let resp = http.get(&url).send().await?;
    if !resp.status().is_success() {
        anyhow::bail!("FRED API returned {}", resp.status());
    }

    let body: FredObservationsResponse = resp.json().await?;
    let mut count = 0usize;

    for obs in &body.observations {
        if obs.value == "." {
            continue;
        }
        let value: f64 = match obs.value.parse() {
            Ok(v) => v,
            Err(_) => continue,
        };
        let date = match NaiveDate::parse_from_str(&obs.date, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => continue,
        };

        sqlx::query(
            r#"
            INSERT INTO macro_rate_spreads (country, spread, date, value, created_at, updated_at)
            VALUES ($1, $2, $3, $4, NOW(), NOW())
            ON CONFLICT (country, spread, date) DO UPDATE SET
                value = EXCLUDED.value,
                updated_at = NOW()
            "#,
        )
        .bind(series.country)
        .bind(series.spread)
        .bind(date)
        .bind(value)
        .execute(pool)
        .await?;

        count += 1;
    }

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_2s10s_spread_correctly() {
        let tenors = vec![("2Y".to_string(), 4.1), ("10Y".to_string(), 3.9)];
        let spread = calculate_spread(&tenors, "2Y", "10Y").unwrap();
        assert!((spread - (-0.2)).abs() < 0.001);
    }

    #[test]
    fn calculates_spread_returns_none_when_tenor_missing() {
        let tenors = vec![("2Y".to_string(), 4.1)];
        assert!(calculate_spread(&tenors, "2Y", "10Y").is_none());
    }
}
