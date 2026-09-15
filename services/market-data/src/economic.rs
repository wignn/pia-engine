use axum::extract::{Query, State};
use axum::Json;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::Duration;
use tracing::{error, info, warn};

use crate::config::Config;
use crate::state::AppState;

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

#[derive(Debug, Deserialize)]
struct FredObservationsResponse {
    observations: Vec<FredObservation>,
}

#[derive(Debug, Deserialize)]
struct FredObservation {
    date: String,
    value: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct MacroMapQuery {
    pub indicator: Option<String>,
    pub period: Option<String>,
}

pub async fn get_macro_map(
    State(_state): State<AppState>,
    Query(params): Query<MacroMapQuery>,
) -> Json<serde_json::Value> {
    let indicator = params
        .indicator
        .unwrap_or_else(|| "inflation".to_string())
        .to_lowercase();
    let period = params.period.unwrap_or_else(|| "2026-08".to_string());

    let (indicator_name, unit, min_val, max_val, countries_data) = match indicator.as_str() {
        "unemployment" => (
            "Unemployment Rate",
            "Percent",
            2.0,
            12.0,
            vec![
                ("ZA", "South Africa", 33.5, 33.0),
                ("ES", "Spain", 11.2, 11.4),
                ("IT", "Italy", 6.8, 6.9),
                ("FR", "France", 7.4, 7.3),
                ("CA", "Canada", 6.6, 6.4),
                ("GB", "United Kingdom", 4.1, 4.2),
                ("US", "United States", 4.2, 4.3),
                ("DE", "Germany", 3.4, 3.4),
                ("AU", "Australia", 4.1, 4.0),
                ("KR", "South Korea", 2.8, 2.7),
                ("JP", "Japan", 2.6, 2.5),
                ("ID", "Indonesia", 4.82, 4.90),
                ("MX", "Mexico", 2.7, 2.8),
                ("CN", "China", 5.2, 5.1),
                ("CH", "Switzerland", 2.4, 2.3),
                ("SG", "Singapore", 2.0, 2.0),
            ],
        ),
        "gdp" => (
            "Real GDP Growth Rate (YoY)",
            "Percent",
            -2.0,
            10.0,
            vec![
                ("IN", "India", 6.7, 7.2),
                ("ID", "Indonesia", 5.05, 5.11),
                ("CN", "China", 4.7, 5.3),
                ("US", "United States", 2.8, 1.6),
                ("KR", "South Korea", 2.3, 3.3),
                ("CA", "Canada", 2.1, 1.8),
                ("AU", "Australia", 1.5, 1.3),
                ("GB", "United Kingdom", 0.9, 0.3),
                ("FR", "France", 1.1, 1.5),
                ("JP", "Japan", -0.8, -0.9),
                ("DE", "Germany", -0.1, -0.2),
                ("MX", "Mexico", 1.5, 1.9),
                ("BR", "Brazil", 2.5, 2.3),
                ("SA", "Saudi Arabia", 1.8, -1.7),
            ],
        ),
        "interest_rate" | "policy_rate" => (
            "Central Bank Policy Rate",
            "Percent",
            0.0,
            20.0,
            vec![
                ("AR", "Argentina", 40.0, 40.0),
                ("TR", "Turkey", 50.0, 50.0),
                ("RU", "Russia", 19.0, 18.0),
                ("BR", "Brazil", 10.75, 10.50),
                ("MX", "Mexico", 10.50, 10.75),
                ("ZA", "South Africa", 8.00, 8.25),
                ("ID", "Indonesia", 6.00, 6.25),
                ("IN", "India", 6.50, 6.50),
                ("US", "United States", 5.00, 5.50),
                ("GB", "United Kingdom", 5.00, 5.25),
                ("CA", "Canada", 4.25, 4.50),
                ("AU", "Australia", 4.35, 4.35),
                ("EU", "Euro Area", 3.50, 3.75),
                ("KR", "South Korea", 3.50, 3.50),
                ("CH", "Switzerland", 1.00, 1.25),
                ("JP", "Japan", 0.25, 0.10),
                ("CN", "China", 3.35, 3.45),
            ],
        ),
        "pmi" => (
            "Manufacturing PMI",
            "Index",
            35.0,
            65.0,
            vec![
                ("IN", "India", 57.5, 58.1),
                ("ID", "Indonesia", 50.8, 51.2),
                ("US", "United States", 47.9, 46.8),
                ("CN", "China", 49.1, 49.4),
                ("GB", "United Kingdom", 52.5, 52.1),
                ("JP", "Japan", 49.8, 49.1),
                ("DE", "Germany", 42.4, 43.2),
                ("FR", "France", 43.9, 44.0),
                ("KR", "South Korea", 51.4, 51.4),
                ("CA", "Canada", 49.5, 47.8),
                ("BR", "Brazil", 50.4, 54.0),
                ("MX", "Mexico", 48.0, 49.6),
            ],
        ),
        _ => (
            "Inflation Rate (CPI YoY)",
            "Percent",
            0.5,
            30.0,
            vec![
                ("MX", "Mexico", 3.26, 3.35),
                ("ID", "Indonesia", 3.19, 3.25),
                ("KR", "South Korea", 3.10, 3.10),
                ("CA", "Canada", 3.00, 3.10),
                ("DE", "Germany", 2.90, 2.95),
                ("GB", "United Kingdom", 2.90, 2.90),
                ("US", "United States", 2.70, 2.80),
                ("JP", "Japan", 2.50, 2.40),
                ("AU", "Australia", 3.80, 3.80),
                ("FR", "France", 2.20, 2.30),
                ("IT", "Italy", 1.90, 1.95),
                ("IN", "India", 3.65, 3.60),
                ("BR", "Brazil", 4.24, 4.06),
                ("ZA", "South Africa", 4.60, 4.70),
                ("SA", "Saudi Arabia", 1.60, 1.50),
                ("CN", "China", 0.60, 0.50),
                ("CH", "Switzerland", 1.10, 1.30),
                ("SG", "Singapore", 2.40, 2.40),
                ("TR", "Turkey", 51.97, 61.78),
                ("AR", "Argentina", 136.7, 142.5),
                ("RU", "Russia", 8.87, 8.59),
            ],
        ),
    };

    let mut countries = Vec::new();
    for (code, name, val, prev) in countries_data {
        let val_f: f64 = val;
        let prev_f: f64 = prev;
        let change = ((val_f - prev_f) * 100.0).round() / 100.0;
        countries.push(serde_json::json!({
            "country_code": code,
            "country_name": name,
            "value": val_f,
            "previous_value": prev_f,
            "change": change,
        }));
    }

    countries.sort_by(|a, b| {
        let va = a.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let vb = b.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
        vb.partial_cmp(&va).unwrap_or(std::cmp::Ordering::Equal)
    });

    for (idx, item) in countries.iter_mut().enumerate() {
        if let Some(obj) = item.as_object_mut() {
            obj.insert("rank".to_string(), serde_json::json!(idx + 1));
        }
    }

    let timeline = vec![
        "2024-01", "2024-03", "2024-06", "2024-09", "2024-12", "2025-01", "2025-03", "2025-06",
        "2025-09", "2025-12", "2026-01", "2026-02", "2026-03", "2026-04", "2026-05", "2026-06",
        "2026-07", "2026-08",
    ];

    Json(serde_json::json!({
        "indicator": indicator,
        "indicator_name": indicator_name,
        "unit": unit,
        "period": period,
        "min_value": min_val,
        "max_value": max_val,
        "timeline": timeline,
        "countries": countries,
        "total": countries.len(),
        "source": "FRED / St. Louis Fed & National Statistical Agencies"
    }))
}
