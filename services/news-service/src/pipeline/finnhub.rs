use chrono::{Duration as ChronoDuration, TimeZone, Utc};
use reqwest::Client;
use serde::Deserialize;
use sqlx::PgPool;
use tracing::{info, warn};

#[derive(Clone)]
pub struct FinnhubClient {
    http: Client,
    token: String,
}

#[derive(Debug, Deserialize)]
struct MarketNewsItem {
    id: i64,
    headline: String,
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    url: String,
    #[serde(default)]
    source: String,
    datetime: i64,
    #[serde(default)]
    category: Option<String>,
}

#[derive(Debug, Deserialize)]
struct EconomicCalendarResponse {
    #[serde(default)]
    economic_calendar: Vec<EconomicCalendarItem>,
}

#[derive(Debug, Deserialize)]
struct EconomicCalendarItem {
    #[serde(default)]
    event: String,
    #[serde(default)]
    country: String,
    #[serde(default)]
    impact: String,
    #[serde(default)]
    unit: Option<String>,
    #[serde(default)]
    actual: Option<f64>,
    #[serde(default)]
    estimate: Option<f64>,
    #[serde(default)]
    previous: Option<f64>,
    time: String,
}

impl FinnhubClient {
    pub fn new(http: Client, token: String) -> Self {
        Self { http, token }
    }

    pub fn enabled(&self) -> bool {
        !self.token.trim().is_empty()
    }

    pub async fn poll_market_news(&self, pool: &PgPool) -> anyhow::Result<usize> {
        if !self.enabled() {
            return Ok(0);
        }

        ensure_source(pool).await?;
        let items = self
            .http
            .get("https://finnhub.io/api/v1/news")
            .query(&[("category", "general"), ("token", self.token.as_str())])
            .send()
            .await?
            .error_for_status()?
            .json::<Vec<MarketNewsItem>>()
            .await?;

        let mut inserted = 0usize;
        for item in items {
            if !is_macro_news(&item) {
                continue;
            }
            let published_at = Utc.timestamp_opt(item.datetime, 0).single();
            let content_hash = format!("finnhub-news-{}", item.id);
            let url = if item.url.trim().is_empty() {
                format!("https://finnhub.io/news/{}", item.id)
            } else {
                item.url
            };
            let count = sqlx::query_scalar::<_, i64>(
                "INSERT INTO news.forex_news_articles (source_id, content_hash, original_url, original_title, original_content, summary, is_processed, processed_at, published_at)
                 VALUES ('feed-finnhub-market-news', $1, $2, $3, $4, $5, TRUE, NOW(), $6)
                 ON CONFLICT (content_hash) DO NOTHING
                 RETURNING 1",
            )
            .bind(content_hash)
            .bind(url)
            .bind(item.headline)
            .bind(item.summary.as_deref())
            .bind(item.summary.as_deref())
            .bind(published_at)
            .fetch_optional(pool)
            .await?
            .unwrap_or(0);
            inserted += count as usize;
        }

        Ok(inserted)
    }

    pub async fn poll_economic_calendar(&self, pool: &PgPool) -> anyhow::Result<usize> {
        if !self.enabled() {
            return Ok(0);
        }

        let from = Utc::now().date_naive();
        let to = (Utc::now() + ChronoDuration::days(7)).date_naive();
        let response = self
            .http
            .get("https://finnhub.io/api/v1/calendar/economic")
            .query(&[
                ("from", from.to_string()),
                ("to", to.to_string()),
                ("token", self.token.clone()),
            ])
            .send()
            .await?
            .error_for_status()?
            .json::<EconomicCalendarResponse>()
            .await?;

        let mut inserted = 0usize;
        for event in response.economic_calendar {
            if event.event.trim().is_empty() || event.time.trim().is_empty() {
                continue;
            }
            let event_hash = calendar_hash(&event);
            let event_time = parse_calendar_time(&event.time);
            let count = sqlx::query_scalar::<_, i64>(
                "INSERT INTO news.economic_calendar_events (source, event_hash, country, event_name, impact, unit, actual, forecast, previous, event_time, raw_payload)
                 VALUES ('finnhub', $1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                 ON CONFLICT (source, event_hash) DO UPDATE SET actual = EXCLUDED.actual, forecast = EXCLUDED.forecast, previous = EXCLUDED.previous, raw_payload = EXCLUDED.raw_payload, updated_at = NOW()
                 RETURNING 1",
            )
            .bind(event_hash)
            .bind(event.country.trim())
            .bind(event.event.trim())
            .bind(empty_to_none(&event.impact))
            .bind(event.unit.as_deref())
            .bind(event.actual)
            .bind(event.estimate)
            .bind(event.previous)
            .bind(event_time)
            .bind(serde_json::to_value(serde_json::json!({
                "country": event.country,
                "event": event.event,
                "impact": event.impact,
                "unit": event.unit,
                "actual": event.actual,
                "estimate": event.estimate,
                "previous": event.previous,
                "time": event.time,
            }))?)
            .fetch_one(pool)
            .await?;
            inserted += count as usize;
        }

        Ok(inserted)
    }
}

pub async fn run_market_news_loop(client: FinnhubClient, pool: PgPool, interval_sec: u64) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_sec.max(600)));
    loop {
        interval.tick().await;
        match client.poll_market_news(&pool).await {
            Ok(inserted) => info!(inserted, "finnhub market news polled"),
            Err(err) => warn!(error = %err, "finnhub market news poll failed"),
        }
    }
}

pub async fn run_economic_calendar_loop(client: FinnhubClient, pool: PgPool, interval_sec: u64) {
    let mut interval =
        tokio::time::interval(std::time::Duration::from_secs(interval_sec.max(1800)));
    loop {
        interval.tick().await;
        match client.poll_economic_calendar(&pool).await {
            Ok(upserted) => info!(upserted, "finnhub economic calendar polled"),
            Err(err) => warn!(error = %err, "finnhub economic calendar poll failed"),
        }
    }
}

async fn ensure_source(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO news.forex_news_sources (id, name, slug, source_type, url, category, poll_interval_sec, priority, is_active, updated_at)
         VALUES ('feed-finnhub-market-news', 'Finnhub Market News', 'finnhub-market-news', 'api', 'https://finnhub.io', 'macro', 900, 90, TRUE, NOW())
         ON CONFLICT (id) DO NOTHING",
    )
    .execute(pool)
    .await?;
    Ok(())
}

fn is_macro_news(item: &MarketNewsItem) -> bool {
    let text = format!(
        "{} {} {} {}",
        item.headline,
        item.summary.as_deref().unwrap_or_default(),
        item.source,
        item.category.as_deref().unwrap_or_default()
    )
    .to_lowercase();

    [
        "fed",
        "inflation",
        "cpi",
        "ppi",
        "jobs",
        "payroll",
        "unemployment",
        "gdp",
        "pmi",
        "rate",
        "yield",
        "treasury",
        "dollar",
        "currency",
        "forex",
        "gold",
        "oil",
        "china",
        "ecb",
        "boj",
        "boe",
        "recession",
        "tariff",
        "geopolitical",
    ]
    .iter()
    .any(|keyword| text.contains(keyword))
}

fn calendar_hash(event: &EconomicCalendarItem) -> String {
    let raw = format!(
        "{}|{}|{}|{}",
        event.time.trim(),
        event.country.trim(),
        event.event.trim(),
        event.impact.trim()
    );
    format!("finnhub-economic-{:x}", md5_like(&raw))
}

fn parse_calendar_time(value: &str) -> Option<chrono::DateTime<Utc>> {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .ok()
        .or_else(|| {
            chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S")
                .ok()
                .map(|dt| dt.and_utc())
        })
}

fn empty_to_none(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn md5_like(value: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}
