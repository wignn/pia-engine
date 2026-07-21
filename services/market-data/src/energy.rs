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
pub struct EnergySeries {
    pub id: String,
    pub source: String,
    pub name: String,
    pub commodity: String,
    pub unit: String,
    pub frequency: String,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EnergyObservation {
    pub series_id: String,
    pub date: NaiveDate,
    pub value: f64,
    pub raw_json: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct EnergyDashboardItem {
    pub series_id: String,
    pub name: String,
    pub commodity: String,
    pub unit: String,
    pub frequency: String,
    pub latest_date: Option<NaiveDate>,
    pub latest_value: Option<f64>,
    pub previous_date: Option<NaiveDate>,
    pub previous_value: Option<f64>,
    pub wow_change: Option<f64>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct SeriesListQuery {
    pub commodity: Option<String>,
    pub active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct SeriesObsQuery {
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub limit: Option<i64>,
}

pub fn calculate_wow_change(latest: f64, previous: f64) -> f64 {
    latest - previous
}

pub async fn list_series(
    State(state): State<AppState>,
    Query(params): Query<SeriesListQuery>,
) -> Json<serde_json::Value> {
    let result = query_series(&state.db, params.commodity.as_deref(), params.active).await;

    match result {
        Ok(data) => Json(serde_json::json!({ "data": data })),
        Err(err) => {
            error!(error = %err, "failed to list energy series");
            Json(serde_json::json!({ "error": "internal server error" }))
        }
    }
}

pub async fn get_series_observations(
    State(state): State<AppState>,
    Path(series_id): Path<String>,
    Query(params): Query<SeriesObsQuery>,
) -> Json<serde_json::Value> {
    let limit = params.limit.unwrap_or(100).clamp(1, 1000);

    let series = match query_series_by_id(&state.db, &series_id).await {
        Ok(Some(s)) => s,
        Ok(None) => return Json(serde_json::json!({ "error": "series not found" })),
        Err(err) => {
            error!(error = %err, series_id = %series_id, "failed to query energy series");
            return Json(serde_json::json!({ "error": "internal server error" }));
        }
    };

    let obs = match query_observations(&state.db, &series_id, params.from, params.to, limit).await {
        Ok(o) => o,
        Err(err) => {
            error!(error = %err, series_id = %series_id, "failed to query energy observations");
            return Json(serde_json::json!({ "error": "internal server error" }));
        }
    };

    Json(serde_json::json!({
        "series": series,
        "observations": obs
    }))
}

pub async fn energy_dashboard(State(state): State<AppState>) -> Json<serde_json::Value> {
    let result = build_dashboard(&state.db).await;

    match result {
        Ok(items) => {
            let updated_at = items.iter().filter_map(|i| i.updated_at).max();
            Json(serde_json::json!({
                "enabled": state.config.has_eia(),
                "items": items,
                "updated_at": updated_at,
            }))
        }
        Err(err) => {
            error!(error = %err, "failed to fetch energy dashboard");
            Json(serde_json::json!({ "error": "internal server error" }))
        }
    }
}

async fn query_series(
    pool: &PgPool,
    commodity: Option<&str>,
    active: Option<bool>,
) -> Result<Vec<EnergySeries>, sqlx::Error> {
    sqlx::query_as::<_, EnergySeries>(
        r#"
        SELECT id, source, name, commodity, unit, frequency, active, created_at, updated_at
        FROM energy_series
        WHERE ($1::text IS NULL OR commodity = $1)
          AND ($2::boolean IS NULL OR active = $2)
        ORDER BY commodity, name
        "#,
    )
    .bind(commodity)
    .bind(active)
    .fetch_all(pool)
    .await
}

async fn query_series_by_id(pool: &PgPool, id: &str) -> Result<Option<EnergySeries>, sqlx::Error> {
    sqlx::query_as::<_, EnergySeries>(
        r#"
        SELECT id, source, name, commodity, unit, frequency, active, created_at, updated_at
        FROM energy_series
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

async fn query_observations(
    pool: &PgPool,
    series_id: &str,
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
    limit: i64,
) -> Result<Vec<EnergyObservation>, sqlx::Error> {
    sqlx::query_as::<_, EnergyObservation>(
        r#"
        SELECT series_id, date, value, raw_json, created_at, updated_at
        FROM energy_observations
        WHERE series_id = $1
          AND ($2::date IS NULL OR date >= $2)
          AND ($3::date IS NULL OR date <= $3)
        ORDER BY date DESC
        LIMIT $4
        "#,
    )
    .bind(series_id)
    .bind(from)
    .bind(to)
    .bind(limit)
    .fetch_all(pool)
    .await
}

async fn build_dashboard(pool: &PgPool) -> Result<Vec<EnergyDashboardItem>, sqlx::Error> {
    let series_list = query_series(pool, None, Some(true)).await?;
    let mut dashboard_items = Vec::new();

    for s in series_list {
        let obs = query_observations(pool, &s.id, None, None, 2).await?;
        let latest = obs.first();
        let previous = obs.get(1);

        let latest_date = latest.map(|o| o.date);
        let latest_value = latest.map(|o| o.value);
        let previous_date = previous.map(|o| o.date);
        let previous_value = previous.map(|o| o.value);

        let wow_change = match (latest_value, previous_value) {
            (Some(l), Some(p)) => Some(calculate_wow_change(l, p)),
            _ => None,
        };

        let updated_at = latest.map(|o| o.updated_at).unwrap_or(s.updated_at);

        dashboard_items.push(EnergyDashboardItem {
            series_id: s.id,
            name: s.name,
            commodity: s.commodity,
            unit: s.unit,
            frequency: s.frequency,
            latest_date,
            latest_value,
            previous_date,
            previous_value,
            wow_change,
            updated_at: Some(updated_at),
        });
    }

    Ok(dashboard_items)
}

// --- EIA Background Sync ---

pub struct PredefinedEnergySeries {
    pub id: &'static str,
    pub name: &'static str,
    pub commodity: &'static str,
    pub unit: &'static str,
    pub frequency: &'static str,
    pub eia_route: &'static str,
    pub facet_series: &'static str,
}

pub const PREDEFINED_SERIES: &[PredefinedEnergySeries] = &[
    PredefinedEnergySeries {
        id: "PET.WCRSTUS1.W",
        name: "U.S. Ending Stocks of Crude Oil",
        commodity: "crude_oil",
        unit: "thousand barrels",
        frequency: "weekly",
        eia_route: "petroleum/stoc/wstk",
        facet_series: "WCRSTUS1",
    },
    PredefinedEnergySeries {
        id: "PET.WRPUPUS2.W",
        name: "U.S. Finished Motor Gasoline Total Products Supplied",
        commodity: "gasoline",
        unit: "thousand barrels per day",
        frequency: "weekly",
        eia_route: "petroleum/cons/wpsup",
        facet_series: "WRPUPUS2",
    },
    PredefinedEnergySeries {
        id: "PET.WDISTUS1.W",
        name: "U.S. Ending Stocks of Distillate Fuel Oil",
        commodity: "distillate",
        unit: "thousand barrels",
        frequency: "weekly",
        eia_route: "petroleum/stoc/wstk",
        facet_series: "WDISTUS1",
    },
    PredefinedEnergySeries {
        id: "NG.NW2_EPG0_SWO_R48_BCF.W",
        name: "Lower 48 States Natural Gas Working Storage",
        commodity: "natural_gas",
        unit: "billion cubic feet",
        frequency: "weekly",
        eia_route: "natural-gas/stor/wk",
        facet_series: "NW2_EPG0_SWO_R48_BCF",
    },
    PredefinedEnergySeries {
        id: "PET.WCREXPUS2.W",
        name: "U.S. Exports of Crude Oil",
        commodity: "crude_oil",
        unit: "thousand barrels per day",
        frequency: "weekly",
        eia_route: "petroleum/move/wkly",
        facet_series: "WCREXPUS2",
    },
];

pub async fn run_energy_sync(config: Config, pool: PgPool) {
    if !config.has_eia() {
        warn!("EIA_API_KEY not set, EIA energy data sync disabled");
        return;
    }

    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("ATLSD-MarketData/1.0")
        .build()
        .unwrap_or_default();

    info!(
        refresh_sec = config.eia_sync_sec,
        series_count = PREDEFINED_SERIES.len(),
        "EIA energy data sync started"
    );

    loop {
        sync_all_energy(&config, &pool, &http).await;
        tokio::time::sleep(Duration::from_secs(config.eia_sync_sec)).await;
    }
}

async fn sync_all_energy(config: &Config, pool: &PgPool, http: &reqwest::Client) {
    info!("starting EIA energy data sync");
    let mut success = 0u32;
    let mut failed = 0u32;

    for series in PREDEFINED_SERIES {
        if let Err(err) = upsert_energy_series_metadata(pool, series).await {
            error!(series_id = series.id, error = %err, "failed to upsert energy series metadata");
        }

        match sync_single_series(config, pool, http, series).await {
            Ok(count) => {
                success += 1;
                if count > 0 {
                    info!(
                        series_id = series.id,
                        observations = count,
                        "synced EIA energy series"
                    );
                }
            }
            Err(err) => {
                failed += 1;
                error!(series_id = series.id, error = %err, "EIA energy series sync failed");
            }
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    info!(success, failed, "EIA energy sync complete");
}

async fn upsert_energy_series_metadata(
    pool: &PgPool,
    series: &PredefinedEnergySeries,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO energy_series (id, source, name, commodity, unit, frequency, active, created_at, updated_at)
        VALUES ($1, 'eia', $2, $3, $4, $5, true, NOW(), NOW())
        ON CONFLICT (id) DO UPDATE SET
            name = EXCLUDED.name,
            commodity = EXCLUDED.commodity,
            unit = EXCLUDED.unit,
            frequency = EXCLUDED.frequency,
            updated_at = NOW()
        "#,
    )
    .bind(series.id)
    .bind(series.name)
    .bind(series.commodity)
    .bind(series.unit)
    .bind(series.frequency)
    .execute(pool)
    .await?;

    Ok(())
}

async fn sync_single_series(
    config: &Config,
    pool: &PgPool,
    http: &reqwest::Client,
    series: &PredefinedEnergySeries,
) -> anyhow::Result<usize> {
    let route_url = format!(
        "https://api.eia.gov/v2/{}/data/?api_key={}&frequency=weekly&data[0]=value&facets[series][]={}&sort[0][column]=period&sort[0][direction]=desc&length=100",
        series.eia_route, config.eia_api_key, series.facet_series
    );

    let items = match fetch_eia_items(http, &route_url).await {
        Ok(items) if !items.is_empty() => items,
        _ => {
            let seq_url = format!(
                "https://api.eia.gov/v2/series/data/?api_key={}&sequence_id={}&sort[0][column]=period&sort[0][direction]=desc&length=100",
                config.eia_api_key, series.id
            );
            fetch_eia_items(http, &seq_url).await?
        }
    };

    let mut count = 0usize;
    for item in items {
        let period_str = match item.get("period").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => continue,
        };

        let date = match NaiveDate::parse_from_str(period_str, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => match NaiveDate::parse_from_str(&format!("{}-01", period_str), "%Y-%m-%d") {
                Ok(d) => d,
                Err(_) => continue,
            },
        };

        let val_raw = match item.get("value") {
            Some(v) => v,
            None => continue,
        };

        let value = match parse_f64_val(val_raw) {
            Some(v) => v,
            None => continue,
        };

        sqlx::query(
            r#"
            INSERT INTO energy_observations (series_id, date, value, raw_json, created_at, updated_at)
            VALUES ($1, $2, $3, $4, NOW(), NOW())
            ON CONFLICT (series_id, date) DO UPDATE SET
                value = EXCLUDED.value,
                raw_json = EXCLUDED.raw_json,
                updated_at = NOW()
            "#,
        )
        .bind(series.id)
        .bind(date)
        .bind(value)
        .bind(&item)
        .execute(pool)
        .await?;

        count += 1;
    }

    Ok(count)
}

async fn fetch_eia_items(
    http: &reqwest::Client,
    url: &str,
) -> anyhow::Result<Vec<serde_json::Value>> {
    let resp = http.get(url).send().await?;
    if !resp.status().is_success() {
        anyhow::bail!("EIA API returned HTTP status {}", resp.status());
    }

    let json_body: serde_json::Value = resp.json().await?;
    let items = json_body
        .get("response")
        .and_then(|r| r.get("data"))
        .and_then(|d| d.as_array())
        .cloned()
        .unwrap_or_default();

    Ok(items)
}

fn parse_f64_val(val: &serde_json::Value) -> Option<f64> {
    match val {
        serde_json::Value::Number(n) => n.as_f64(),
        serde_json::Value::String(s) => s.trim().parse::<f64>().ok(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_wow_change_correctly() {
        let latest = 420.5;
        let previous = 415.0;
        let diff = calculate_wow_change(latest, previous);
        assert!((diff - 5.5).abs() < 0.001);
    }

    #[test]
    fn parses_f64_val_from_json_number_and_string() {
        assert_eq!(parse_f64_val(&serde_json::json!(420.5)), Some(420.5));
        assert_eq!(parse_f64_val(&serde_json::json!("415.0")), Some(415.0));
        assert_eq!(parse_f64_val(&serde_json::json!(null)), None);
        assert_eq!(parse_f64_val(&serde_json::json!("invalid")), None);
    }
}
