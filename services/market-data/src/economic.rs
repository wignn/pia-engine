use axum::extract::{Query, State};
use axum::Json;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::Duration;
use tracing::{error, info, warn};

use crate::config::Config;
use crate::state::AppState;

// --- FRED Series Registry ---

#[derive(Clone)]
#[allow(dead_code)]
struct FredSeries {
    id: &'static str,
    title: &'static str,
    country: &'static str,
    category: &'static str,
    units: &'static str,
    frequency: &'static str,
}

const SERIES_REGISTRY: &[FredSeries] = &[
    // US GDP
    FredSeries {
        id: "GDP",
        title: "Gross Domestic Product",
        country: "US",
        category: "gdp",
        units: "Billions of Dollars",
        frequency: "Quarterly",
    },
    FredSeries {
        id: "GDPC1",
        title: "Real GDP",
        country: "US",
        category: "gdp",
        units: "Billions of Chained 2017 Dollars",
        frequency: "Quarterly",
    },
    FredSeries {
        id: "A191RL1Q225SBEA",
        title: "Real GDP Growth Rate",
        country: "US",
        category: "gdp",
        units: "Percent",
        frequency: "Quarterly",
    },
    // International GDP
    FredSeries {
        id: "CLVMNACSCAB1GQEU272020",
        title: "EU Real GDP",
        country: "EU",
        category: "gdp",
        units: "Millions of Chained 2010 Euros",
        frequency: "Quarterly",
    },
    FredSeries {
        id: "NAEXKP01GBQ189S",
        title: "UK Real GDP Growth Rate",
        country: "GB",
        category: "gdp",
        units: "Percent",
        frequency: "Quarterly",
    },
    FredSeries {
        id: "NAEXKP01JPQ189S",
        title: "Japan Real GDP Growth Rate",
        country: "JP",
        category: "gdp",
        units: "Percent",
        frequency: "Quarterly",
    },
    FredSeries {
        id: "NAEXKP01CNQ189S",
        title: "China Real GDP Growth Rate",
        country: "CN",
        category: "gdp",
        units: "Percent",
        frequency: "Quarterly",
    },
    // US PMI & Manufacturing
    FredSeries {
        id: "MANEMP",
        title: "Manufacturing Employment",
        country: "US",
        category: "pmi",
        units: "Thousands of Persons",
        frequency: "Monthly",
    },
    FredSeries {
        id: "INDPRO",
        title: "Industrial Production Index",
        country: "US",
        category: "pmi",
        units: "Index 2017=100",
        frequency: "Monthly",
    },
    FredSeries {
        id: "IPMAN",
        title: "Manufacturing Industrial Production",
        country: "US",
        category: "pmi",
        units: "Index 2017=100",
        frequency: "Monthly",
    },
    // US Inflation
    FredSeries {
        id: "CPIAUCSL",
        title: "Consumer Price Index (All Urban)",
        country: "US",
        category: "inflation",
        units: "Index 1982-1984=100",
        frequency: "Monthly",
    },
    FredSeries {
        id: "CPILFESL",
        title: "Core CPI (Ex Food & Energy)",
        country: "US",
        category: "inflation",
        units: "Index 1982-1984=100",
        frequency: "Monthly",
    },
    FredSeries {
        id: "PCEPI",
        title: "PCE Price Index",
        country: "US",
        category: "inflation",
        units: "Index 2017=100",
        frequency: "Monthly",
    },
    FredSeries {
        id: "PPIFIS",
        title: "Producer Price Index (Final Demand)",
        country: "US",
        category: "inflation",
        units: "Index Nov 2009=100",
        frequency: "Monthly",
    },
    // International Inflation
    FredSeries {
        id: "CP0000GBM086NEST",
        title: "UK CPI",
        country: "GB",
        category: "inflation",
        units: "Index 2015=100",
        frequency: "Monthly",
    },
    FredSeries {
        id: "CP0000JPM086NEST",
        title: "Japan CPI",
        country: "JP",
        category: "inflation",
        units: "Index 2015=100",
        frequency: "Monthly",
    },
    FredSeries {
        id: "CP0000EZ19M086NEST",
        title: "Euro Area CPI",
        country: "EU",
        category: "inflation",
        units: "Index 2015=100",
        frequency: "Monthly",
    },
    // US Employment
    FredSeries {
        id: "UNRATE",
        title: "Unemployment Rate",
        country: "US",
        category: "employment",
        units: "Percent",
        frequency: "Monthly",
    },
    FredSeries {
        id: "PAYEMS",
        title: "Non-Farm Payrolls",
        country: "US",
        category: "employment",
        units: "Thousands of Persons",
        frequency: "Monthly",
    },
    FredSeries {
        id: "ICSA",
        title: "Initial Jobless Claims",
        country: "US",
        category: "employment",
        units: "Number",
        frequency: "Weekly",
    },
    FredSeries {
        id: "CCSA",
        title: "Continued Jobless Claims",
        country: "US",
        category: "employment",
        units: "Number",
        frequency: "Weekly",
    },
    // International Employment
    FredSeries {
        id: "LRHUTTTTGBM156S",
        title: "UK Unemployment Rate",
        country: "GB",
        category: "employment",
        units: "Percent",
        frequency: "Monthly",
    },
    FredSeries {
        id: "LRHUTTTTJPM156S",
        title: "Japan Unemployment Rate",
        country: "JP",
        category: "employment",
        units: "Percent",
        frequency: "Monthly",
    },
    FredSeries {
        id: "LRHUTTTTEZM156S",
        title: "Euro Area Unemployment Rate",
        country: "EU",
        category: "employment",
        units: "Percent",
        frequency: "Monthly",
    },
];

// --- Models ---

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct EconomicObservation {
    pub series_id: String,
    pub title: String,
    pub country: String,
    pub category: String,
    pub observation_date: NaiveDate,
    pub value: Option<f64>,
    pub unit: String,
    pub frequency: String,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct EconomicResponse {
    pub data: Vec<EconomicObservation>,
    pub meta: PaginationMeta,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct PaginationMeta {
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Serialize)]
pub struct CountryInfo {
    pub code: &'static str,
    pub name: &'static str,
}

#[derive(Debug, Serialize)]
pub struct CategoryInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
}

// --- Query params ---

#[derive(Debug, Deserialize)]
pub struct IndicatorsQuery {
    pub country: Option<String>,
    pub category: Option<String>,
    pub series_id: Option<String>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct SeriesQuery {
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct LatestQuery {
    pub country: Option<String>,
    pub category: Option<String>,
}

// --- Handlers ---

pub async fn list_indicators(
    State(state): State<AppState>,
    Query(params): Query<IndicatorsQuery>,
) -> Json<serde_json::Value> {
    let limit = params.limit.unwrap_or(50).clamp(1, 500);
    let offset = params.offset.unwrap_or(0).max(0);

    let result = query_indicators(
        &state.db,
        params.country.as_deref(),
        params.category.as_deref(),
        params.series_id.as_deref(),
        params.from,
        params.to,
        limit,
        offset,
    )
    .await;

    match result {
        Ok((data, total)) => Json(serde_json::json!({
            "data": data,
            "meta": { "total": total, "limit": limit, "offset": offset }
        })),
        Err(err) => {
            error!(error = %err, "failed to query economic indicators");
            Json(serde_json::json!({ "error": "internal server error" }))
        }
    }
}

pub async fn get_series(
    State(state): State<AppState>,
    axum::extract::Path(series_id): axum::extract::Path<String>,
    Query(params): Query<SeriesQuery>,
) -> Json<serde_json::Value> {
    let limit = params.limit.unwrap_or(100).clamp(1, 500);

    let result = query_indicators(
        &state.db,
        None,
        None,
        Some(&series_id),
        params.from,
        params.to,
        limit,
        0,
    )
    .await;

    match result {
        Ok((data, total)) => Json(serde_json::json!({
            "data": data,
            "meta": { "total": total, "limit": limit, "offset": 0 }
        })),
        Err(err) => {
            error!(error = %err, series_id = %series_id, "failed to query series");
            Json(serde_json::json!({ "error": "internal server error" }))
        }
    }
}

pub async fn latest_indicators(
    State(state): State<AppState>,
    Query(params): Query<LatestQuery>,
) -> Json<serde_json::Value> {
    let result = query_latest(
        &state.db,
        params.country.as_deref(),
        params.category.as_deref(),
    )
    .await;

    match result {
        Ok(data) => Json(serde_json::json!({ "data": data })),
        Err(err) => {
            error!(error = %err, "failed to query latest indicators");
            Json(serde_json::json!({ "error": "internal server error" }))
        }
    }
}

pub async fn list_countries() -> Json<serde_json::Value> {
    let countries = vec![
        CountryInfo {
            code: "US",
            name: "United States",
        },
        CountryInfo {
            code: "GB",
            name: "United Kingdom",
        },
        CountryInfo {
            code: "JP",
            name: "Japan",
        },
        CountryInfo {
            code: "EU",
            name: "Euro Area",
        },
        CountryInfo {
            code: "CN",
            name: "China",
        },
    ];
    Json(serde_json::json!({ "data": countries }))
}

pub async fn list_categories() -> Json<serde_json::Value> {
    let categories = vec![
        CategoryInfo {
            id: "gdp",
            name: "GDP & Growth",
            description: "Gross Domestic Product and growth rates",
        },
        CategoryInfo {
            id: "pmi",
            name: "PMI & Manufacturing",
            description: "Purchasing Managers Index and industrial production",
        },
        CategoryInfo {
            id: "inflation",
            name: "Inflation & CPI",
            description: "Consumer and producer price indices",
        },
        CategoryInfo {
            id: "employment",
            name: "Employment",
            description: "Unemployment rates, payrolls, and jobless claims",
        },
    ];
    Json(serde_json::json!({ "data": categories }))
}

// --- DB Queries ---

#[allow(clippy::too_many_arguments)]
async fn query_indicators(
    pool: &PgPool,
    country: Option<&str>,
    category: Option<&str>,
    series_id: Option<&str>,
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
    limit: i64,
    offset: i64,
) -> Result<(Vec<EconomicObservation>, i64), sqlx::Error> {
    let rows = sqlx::query_as::<_, EconomicObservation>(
        r#"
        SELECT
            ms.id as series_id,
            ms.title,
            COALESCE(
                CASE
                    WHEN ms.id LIKE '%GB%' OR ms.id LIKE '%UK%' THEN 'GB'
                    WHEN ms.id LIKE '%JP%' OR ms.id LIKE '%JPN%' THEN 'JP'
                    WHEN ms.id LIKE '%EZ%' OR ms.id LIKE '%EU%' THEN 'EU'
                    WHEN ms.id LIKE '%CN%' OR ms.id LIKE '%CHN%' THEN 'CN'
                    ELSE 'US'
                END,
                'US'
            ) as country,
            ms.category,
            mo.observation_date,
            mo.value,
            COALESCE(ms.units, '') as unit,
            COALESCE(ms.frequency, '') as frequency
        FROM macro_series ms
        JOIN macro_observations mo ON mo.series_id = ms.id
        WHERE ms.provider = 'fred'
            AND ($1::text IS NULL OR
                CASE
                    WHEN ms.id LIKE '%GB%' OR ms.id LIKE '%UK%' THEN 'GB'
                    WHEN ms.id LIKE '%JP%' OR ms.id LIKE '%JPN%' THEN 'JP'
                    WHEN ms.id LIKE '%EZ%' OR ms.id LIKE '%EU%' THEN 'EU'
                    WHEN ms.id LIKE '%CN%' OR ms.id LIKE '%CHN%' THEN 'CN'
                    ELSE 'US'
                END = $1)
            AND ($2::text IS NULL OR ms.category = $2)
            AND ($3::text IS NULL OR ms.id = $3)
            AND ($4::date IS NULL OR mo.observation_date >= $4)
            AND ($5::date IS NULL OR mo.observation_date <= $5)
        ORDER BY mo.observation_date DESC
        LIMIT $6 OFFSET $7
        "#,
    )
    .bind(country)
    .bind(category)
    .bind(series_id)
    .bind(from)
    .bind(to)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let total: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)::bigint
        FROM macro_series ms
        JOIN macro_observations mo ON mo.series_id = ms.id
        WHERE ms.provider = 'fred'
            AND ($1::text IS NULL OR
                CASE
                    WHEN ms.id LIKE '%GB%' OR ms.id LIKE '%UK%' THEN 'GB'
                    WHEN ms.id LIKE '%JP%' OR ms.id LIKE '%JPN%' THEN 'JP'
                    WHEN ms.id LIKE '%EZ%' OR ms.id LIKE '%EU%' THEN 'EU'
                    WHEN ms.id LIKE '%CN%' OR ms.id LIKE '%CHN%' THEN 'CN'
                    ELSE 'US'
                END = $1)
            AND ($2::text IS NULL OR ms.category = $2)
            AND ($3::text IS NULL OR ms.id = $3)
            AND ($4::date IS NULL OR mo.observation_date >= $4)
            AND ($5::date IS NULL OR mo.observation_date <= $5)
        "#,
    )
    .bind(country)
    .bind(category)
    .bind(series_id)
    .bind(from)
    .bind(to)
    .fetch_one(pool)
    .await?;

    Ok((rows, total.0))
}

async fn query_latest(
    pool: &PgPool,
    country: Option<&str>,
    category: Option<&str>,
) -> Result<Vec<EconomicObservation>, sqlx::Error> {
    sqlx::query_as::<_, EconomicObservation>(
        r#"
        SELECT DISTINCT ON (ms.id)
            ms.id as series_id,
            ms.title,
            COALESCE(
                CASE
                    WHEN ms.id LIKE '%GB%' OR ms.id LIKE '%UK%' THEN 'GB'
                    WHEN ms.id LIKE '%JP%' OR ms.id LIKE '%JPN%' THEN 'JP'
                    WHEN ms.id LIKE '%EZ%' OR ms.id LIKE '%EU%' THEN 'EU'
                    WHEN ms.id LIKE '%CN%' OR ms.id LIKE '%CHN%' THEN 'CN'
                    ELSE 'US'
                END,
                'US'
            ) as country,
            ms.category,
            mo.observation_date,
            mo.value,
            COALESCE(ms.units, '') as unit,
            COALESCE(ms.frequency, '') as frequency
        FROM macro_series ms
        JOIN macro_observations mo ON mo.series_id = ms.id
        WHERE ms.provider = 'fred'
            AND ($1::text IS NULL OR
                CASE
                    WHEN ms.id LIKE '%GB%' OR ms.id LIKE '%UK%' THEN 'GB'
                    WHEN ms.id LIKE '%JP%' OR ms.id LIKE '%JPN%' THEN 'JP'
                    WHEN ms.id LIKE '%EZ%' OR ms.id LIKE '%EU%' THEN 'EU'
                    WHEN ms.id LIKE '%CN%' OR ms.id LIKE '%CHN%' THEN 'CN'
                    ELSE 'US'
                END = $1)
            AND ($2::text IS NULL OR ms.category = $2)
        ORDER BY ms.id, mo.observation_date DESC
        "#,
    )
    .bind(country)
    .bind(category)
    .fetch_all(pool)
    .await
}

// --- Background Job: FRED Sync ---

pub async fn run_sync(config: Config, pool: PgPool) {
    if !config.has_fred() {
        warn!("FRED_API_KEY not set, economic data sync disabled");
        return;
    }

    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap_or_default();

    info!(
        refresh_sec = config.economic_refresh_sec,
        series_count = SERIES_REGISTRY.len(),
        "economic data sync started"
    );

    loop {
        sync_all_series(&config, &pool, &http).await;
        tokio::time::sleep(Duration::from_secs(config.economic_refresh_sec)).await;
    }
}

async fn sync_all_series(config: &Config, pool: &PgPool, http: &reqwest::Client) {
    info!("starting FRED data sync");
    let mut success = 0u32;
    let mut failed = 0u32;

    for series in SERIES_REGISTRY {
        match sync_series(config, pool, http, series).await {
            Ok(count) => {
                success += 1;
                if count > 0 {
                    info!(series_id = series.id, observations = count, "synced");
                }
            }
            Err(err) => {
                failed += 1;
                error!(series_id = series.id, error = %err, "sync failed");
            }
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    info!(success, failed, "FRED sync complete");
}

async fn sync_series(
    config: &Config,
    pool: &PgPool,
    http: &reqwest::Client,
    series: &FredSeries,
) -> anyhow::Result<usize> {
    // Upsert series metadata
    sqlx::query(
        r#"
        INSERT INTO macro_series (id, provider, title, category, units, frequency, last_synced_at)
        VALUES ($1, 'fred', $2, $3, $4, $5, NOW())
        ON CONFLICT (id) DO UPDATE SET
            title = EXCLUDED.title,
            category = EXCLUDED.category,
            units = EXCLUDED.units,
            frequency = EXCLUDED.frequency,
            last_synced_at = NOW(),
            updated_at = NOW()
        "#,
    )
    .bind(series.id)
    .bind(series.title)
    .bind(series.category)
    .bind(series.units)
    .bind(series.frequency)
    .execute(pool)
    .await?;

    // Fetch observations from FRED
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
            INSERT INTO macro_observations (series_id, observation_date, value, raw_value)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (series_id, observation_date) DO UPDATE SET
                value = EXCLUDED.value,
                raw_value = EXCLUDED.raw_value,
                updated_at = NOW()
            "#,
        )
        .bind(series.id)
        .bind(date)
        .bind(value)
        .bind(&obs.value)
        .execute(pool)
        .await?;

        count += 1;
    }

    // Update observation range on series
    sqlx::query(
        r#"
        UPDATE macro_series SET
            observation_start = (SELECT MIN(observation_date) FROM macro_observations WHERE series_id = $1),
            observation_end = (SELECT MAX(observation_date) FROM macro_observations WHERE series_id = $1),
            updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(series.id)
    .execute(pool)
    .await?;

    Ok(count)
}

// --- FRED API response types ---

#[derive(Debug, Deserialize)]
struct FredObservationsResponse {
    observations: Vec<FredObservation>,
}

#[derive(Debug, Deserialize)]
struct FredObservation {
    date: String,
    value: String,
}
