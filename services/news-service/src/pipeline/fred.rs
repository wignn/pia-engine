use chrono::{NaiveDate, Utc};
use reqwest::Client;
use serde::Deserialize;
use sqlx::PgPool;
use tracing::{info, warn};

const DEFAULT_SERIES: &[(&str, &str)] = &[
    ("DGS10", "rates"),
    ("DGS2", "rates"),
    ("T10Y2Y", "rates"),
    ("CPIAUCSL", "inflation"),
    ("CPILFESL", "inflation"),
    ("PCEPI", "inflation"),
    ("PCEPILFE", "inflation"),
    ("T5YIE", "inflation"),
    ("T10YIE", "inflation"),
    ("UNRATE", "labor"),
    ("PAYEMS", "labor"),
    ("ICSA", "labor"),
    ("JTSJOL", "labor"),
    ("GDP", "growth"),
    ("GDPC1", "growth"),
    ("INDPRO", "growth"),
    ("RSAFS", "growth"),
    ("UMCSENT", "growth"),
    ("DTWEXBGS", "dollar_liquidity"),
    ("WALCL", "dollar_liquidity"),
    ("M2SL", "dollar_liquidity"),
    ("BAMLH0A0HYM2", "credit_stress"),
    ("BAMLC0A0CM", "credit_stress"),
    ("DCOILWTICO", "commodities"),
    ("DCOILBRENTEU", "commodities"),
    ("HOUST", "housing"),
    ("CSUSHPINSA", "housing"),
    ("MORTGAGE30US", "housing"),
    ("FEDFUNDS", "fed_policy"),
    ("DFF", "fed_policy"),
    ("DFEDTARU", "fed_policy"),
    ("DFEDTARL", "fed_policy"),
];

#[derive(Clone)]
pub struct FredClient {
    http: Client,
    api_key: String,
    series: Vec<SeriesConfig>,
}

#[derive(Clone)]
struct SeriesConfig {
    id: String,
    category: String,
}

#[derive(Debug, Deserialize)]
struct SeriesResponse {
    #[serde(default)]
    seriess: Vec<SeriesMetadata>,
}

#[derive(Debug, Deserialize)]
struct SeriesMetadata {
    id: String,
    title: String,
    #[serde(default)]
    units: Option<String>,
    #[serde(default)]
    frequency: Option<String>,
    #[serde(default)]
    seasonal_adjustment: Option<String>,
    #[serde(default)]
    observation_start: Option<String>,
    #[serde(default)]
    observation_end: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ObservationsResponse {
    #[serde(default)]
    observations: Vec<Observation>,
}

#[derive(Debug, Deserialize)]
struct Observation {
    date: String,
    value: String,
}

impl FredClient {
    pub fn new(http: Client, api_key: String, configured_series: Vec<String>) -> Self {
        let series = if configured_series.is_empty() {
            DEFAULT_SERIES
                .iter()
                .map(|(id, category)| SeriesConfig {
                    id: (*id).to_string(),
                    category: (*category).to_string(),
                })
                .collect()
        } else {
            configured_series
                .into_iter()
                .map(|id| SeriesConfig {
                    category: default_category(&id).to_string(),
                    id,
                })
                .collect()
        };

        Self {
            http,
            api_key,
            series,
        }
    }

    pub fn enabled(&self) -> bool {
        !self.api_key.trim().is_empty() && !self.series.is_empty()
    }

    pub async fn sync_all(&self, pool: &PgPool) -> anyhow::Result<usize> {
        if !self.enabled() {
            return Ok(0);
        }

        let mut upserted = 0usize;
        for series in &self.series {
            match self.sync_series(pool, series).await {
                Ok(count) => upserted += count,
                Err(err) => warn!(series = %series.id, error = %err, "fred series sync failed"),
            }
        }
        crate::pipeline::r#macro::refresh_signals(pool).await?;
        Ok(upserted)
    }

    async fn sync_series(&self, pool: &PgPool, series: &SeriesConfig) -> anyhow::Result<usize> {
        let metadata = self.fetch_metadata(&series.id).await?;
        upsert_series(pool, &metadata, &series.category).await?;

        let observations = self.fetch_observations(&series.id).await?;
        let mut upserted = 0usize;
        for observation in observations {
            let Some(date) = parse_date(&observation.date) else {
                continue;
            };
            let value = parse_fred_value(&observation.value);
            sqlx::query(
                "INSERT INTO macro_observations (series_id, observation_date, value, raw_value)
                 VALUES ($1, $2, $3, $4)
                 ON CONFLICT (series_id, observation_date) DO UPDATE SET value = EXCLUDED.value, raw_value = EXCLUDED.raw_value, updated_at = NOW()",
            )
            .bind(&series.id)
            .bind(date)
            .bind(value)
            .bind(&observation.value)
            .execute(pool)
            .await?;
            upserted += 1;
        }

        sqlx::query(
            "UPDATE macro_series SET last_synced_at = NOW(), updated_at = NOW() WHERE id = $1",
        )
        .bind(&series.id)
        .execute(pool)
        .await?;

        Ok(upserted)
    }

    async fn fetch_metadata(&self, series_id: &str) -> anyhow::Result<SeriesMetadata> {
        let response = self
            .http
            .get("https://api.stlouisfed.org/fred/series")
            .query(&[
                ("series_id", series_id),
                ("api_key", self.api_key.as_str()),
                ("file_type", "json"),
            ])
            .send()
            .await?
            .error_for_status()?
            .json::<SeriesResponse>()
            .await?;

        response
            .seriess
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("FRED series metadata not found: {series_id}"))
    }

    async fn fetch_observations(&self, series_id: &str) -> anyhow::Result<Vec<Observation>> {
        let start = (Utc::now().date_naive() - chrono::Duration::days(370)).to_string();
        let response = self
            .http
            .get("https://api.stlouisfed.org/fred/series/observations")
            .query(&[
                ("series_id", series_id),
                ("api_key", self.api_key.as_str()),
                ("file_type", "json"),
                ("observation_start", start.as_str()),
                ("sort_order", "asc"),
            ])
            .send()
            .await?
            .error_for_status()?
            .json::<ObservationsResponse>()
            .await?;

        Ok(response.observations)
    }
}

pub async fn run_loop(client: FredClient, pool: PgPool, interval_sec: u64) {
    let mut interval =
        tokio::time::interval(std::time::Duration::from_secs(interval_sec.max(21_600)));
    loop {
        interval.tick().await;
        match client.sync_all(&pool).await {
            Ok(upserted) => info!(upserted, "fred macro data synced"),
            Err(err) => warn!(error = %err, "fred macro sync failed"),
        }
    }
}

async fn upsert_series(
    pool: &PgPool,
    metadata: &SeriesMetadata,
    category: &str,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO macro_series (id, provider, title, category, units, frequency, seasonal_adjustment, observation_start, observation_end)
         VALUES ($1, 'fred', $2, $3, $4, $5, $6, $7, $8)
         ON CONFLICT (id) DO UPDATE SET title = EXCLUDED.title, category = EXCLUDED.category, units = EXCLUDED.units, frequency = EXCLUDED.frequency, seasonal_adjustment = EXCLUDED.seasonal_adjustment, observation_start = EXCLUDED.observation_start, observation_end = EXCLUDED.observation_end, updated_at = NOW()",
    )
    .bind(&metadata.id)
    .bind(&metadata.title)
    .bind(category)
    .bind(metadata.units.as_deref())
    .bind(metadata.frequency.as_deref())
    .bind(metadata.seasonal_adjustment.as_deref())
    .bind(metadata.observation_start.as_deref().and_then(parse_date))
    .bind(metadata.observation_end.as_deref().and_then(parse_date))
    .execute(pool)
    .await?;

    Ok(())
}

fn parse_date(value: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").ok()
}

fn parse_fred_value(value: &str) -> Option<f64> {
    if value.trim() == "." {
        None
    } else {
        value.parse::<f64>().ok()
    }
}

fn default_category(series_id: &str) -> &'static str {
    DEFAULT_SERIES
        .iter()
        .find_map(|(id, category)| (*id == series_id).then_some(*category))
        .unwrap_or("macro")
}
