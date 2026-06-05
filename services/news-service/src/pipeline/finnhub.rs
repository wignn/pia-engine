use chrono::{Duration as ChronoDuration, TimeZone, Utc};
use reqwest::Client;
use serde::Deserialize;
use sqlx::PgPool;
use tracing::{info, warn};

const FALLBACK_NEWS_SOURCE: &str = "Market News Wire";

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

fn calendar_geosignal_event_id(event_hash: &str) -> String {
    format!("calendar:finnhub:{event_hash}")
}

fn calendar_severity_score(impact: &str) -> f64 {
    match impact.trim().to_lowercase().as_str() {
        "high" => 0.75,
        "medium" => 0.5,
        "low" => 0.25,
        _ => 0.25,
    }
}

fn calendar_sentiment_score(actual: Option<f64>, forecast: Option<f64>) -> f64 {
    match (actual, forecast) {
        (Some(actual), Some(forecast)) if actual > forecast => 0.25,
        (Some(actual), Some(forecast)) if actual < forecast => -0.25,
        _ => 0.0,
    }
}

fn calendar_affected_assets(country: &str, event_name: &str) -> Vec<String> {
    let country = country.trim().to_lowercase();
    let event = event_name.trim().to_lowercase();
    if event.contains("oil") {
        return vec!["WTI".to_string()];
    }

    let us_event = matches!(country.as_str(), "us" | "usa" | "united states")
        || ["fed", "cpi", "jobs", "payroll", "yield", "rate"]
            .iter()
            .any(|term| event.contains(term));
    if us_event {
        vec!["DXY".to_string(), "US10Y".to_string()]
    } else {
        Vec::new()
    }
}

fn calendar_asset_impact(assets: &[String], severity_score: f64) -> serde_json::Value {
    serde_json::Value::Object(
        assets
            .iter()
            .map(|asset| (asset.clone(), serde_json::json!(severity_score)))
            .collect(),
    )
}

fn calendar_summary(event: &EconomicCalendarItem) -> String {
    format!(
        "{} {} impact; actual {:?}, forecast {:?}, previous {:?}.",
        event.country.trim(),
        event.impact.trim(),
        event.actual,
        event.estimate,
        event.previous
    )
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
            let source_name = clean_news_source(&item.source);
            let headline = clean_news_headline(&item.headline, &source_name);
            let source_id = ensure_news_source(pool, &source_name).await?;
            let url = if item.url.trim().is_empty() {
                format!("https://finnhub.io/news/{}", item.id)
            } else {
                item.url
            };
            let count = sqlx::query_scalar::<_, i64>(
                "INSERT INTO news.forex_news_articles (source_id, content_hash, original_url, original_title, original_content, summary, is_processed, processed_at, published_at)
                 VALUES ($1, $2, $3, $4, $5, $6, TRUE, NOW(), $7)
                 ON CONFLICT (content_hash) DO NOTHING
                 RETURNING 1",
            )
            .bind(source_id)
            .bind(content_hash)
            .bind(url)
            .bind(headline)
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
            let raw_payload = serde_json::to_value(serde_json::json!({
                "country": event.country,
                "event": event.event,
                "impact": event.impact,
                "unit": event.unit,
                "actual": event.actual,
                "estimate": event.estimate,
                "previous": event.previous,
                "time": event.time,
            }))?;
            let mut tx = pool.begin().await?;
            let count = sqlx::query_scalar::<_, i64>(
                "INSERT INTO news.economic_calendar_events (source, event_hash, country, event_name, impact, unit, actual, forecast, previous, event_time, raw_payload)
                 VALUES ('finnhub', $1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                 ON CONFLICT (source, event_hash) DO UPDATE SET actual = EXCLUDED.actual, forecast = EXCLUDED.forecast, previous = EXCLUDED.previous, raw_payload = EXCLUDED.raw_payload, updated_at = NOW()
                 RETURNING 1",
            )
            .bind(&event_hash)
            .bind(event.country.trim())
            .bind(event.event.trim())
            .bind(empty_to_none(&event.impact))
            .bind(event.unit.as_deref())
            .bind(event.actual)
            .bind(event.estimate)
            .bind(event.previous)
            .bind(event_time)
            .bind(raw_payload)
            .fetch_one(&mut *tx)
            .await?;

            let assets = calendar_affected_assets(&event.country, &event.event);
            let severity_score = calendar_severity_score(&event.impact);
            sqlx::query(
                "INSERT INTO news.geosignals (event_id, timestamp, source, source_url, title, summary, category, country, region, location_scope, severity_score, sentiment_score, confidence_score, affected_assets, asset_impact, freshness)
                 VALUES ($1, $2, 'finnhub', NULL, $3, $4, 'macro', $5, NULL, $6, $7, $8, 0.7, $9, $10, 'fresh')
                 ON CONFLICT (event_id) DO UPDATE SET timestamp = EXCLUDED.timestamp, title = EXCLUDED.title, summary = EXCLUDED.summary, country = EXCLUDED.country, location_scope = EXCLUDED.location_scope, severity_score = EXCLUDED.severity_score, sentiment_score = EXCLUDED.sentiment_score, affected_assets = EXCLUDED.affected_assets, asset_impact = EXCLUDED.asset_impact, freshness = EXCLUDED.freshness",
            )
            .bind(calendar_geosignal_event_id(&event_hash))
            .bind(event_time.unwrap_or_else(Utc::now))
            .bind(event.event.trim())
            .bind(calendar_summary(&event))
            .bind(empty_to_none(&event.country))
            .bind(if event.country.trim().is_empty() { "global" } else { "country" })
            .bind(severity_score)
            .bind(calendar_sentiment_score(event.actual, event.estimate))
            .bind(&assets)
            .bind(calendar_asset_impact(&assets, severity_score))
            .execute(&mut *tx)
            .await?;
            tx.commit().await?;

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

async fn ensure_news_source(pool: &PgPool, name: &str) -> anyhow::Result<String> {
    let slug = source_slug(name);
    let id = format!("feed-market-news-{slug}");
    sqlx::query(
        "INSERT INTO news.forex_news_sources (id, name, slug, source_type, url, category, poll_interval_sec, priority, is_active, updated_at)
         VALUES ($1, $2, $3, 'api', '', 'macro', 900, 90, TRUE, NOW())
         ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name, updated_at = NOW()",
    )
    .bind(&id)
    .bind(name)
    .bind(slug)
    .execute(pool)
    .await?;
    Ok(id)
}

fn clean_news_source(source: &str) -> String {
    let source = source.trim();
    if source.is_empty() {
        FALLBACK_NEWS_SOURCE.to_string()
    } else {
        source.to_string()
    }
}

fn clean_news_headline(headline: &str, source: &str) -> String {
    let headline = headline.trim();
    let source = source.trim();
    if source.is_empty() || source == FALLBACK_NEWS_SOURCE {
        return headline.to_string();
    }

    let lower = headline.to_lowercase();
    let source_lower = source.to_lowercase();
    for separator in [" - ", " – ", " — "] {
        let suffix = format!("{separator}{source_lower}");
        if lower.ends_with(&suffix) {
            return headline[..headline.len() - suffix.len()].trim().to_string();
        }
    }

    headline.to_string()
}

fn source_slug(source: &str) -> String {
    let slug: String = source
        .trim()
        .to_lowercase()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect();
    let slug = slug
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if slug.is_empty() {
        "market-news-wire".to_string()
    } else {
        slug
    }
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
    format!("finnhub-economic-{}", stable_hash_prefix(&raw))
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

fn stable_hash_prefix(value: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    let digest = hasher.finalize();
    hex::encode(&digest[..8])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_calendar_geosignal_event_id() {
        assert_eq!(calendar_geosignal_event_id("abc"), "calendar:finnhub:abc");
    }

    #[test]
    fn maps_calendar_impact_to_severity_score() {
        assert_eq!(calendar_severity_score("high"), 0.75);
        assert_eq!(calendar_severity_score("medium"), 0.5);
        assert_eq!(calendar_severity_score("low"), 0.25);
        assert_eq!(calendar_severity_score("unknown"), 0.25);
    }

    #[test]
    fn maps_calendar_actual_vs_forecast_to_sentiment_score() {
        assert_eq!(calendar_sentiment_score(Some(2.0), Some(1.0)), 0.25);
        assert_eq!(calendar_sentiment_score(Some(1.0), Some(2.0)), -0.25);
        assert_eq!(calendar_sentiment_score(Some(1.0), Some(1.0)), 0.0);
        assert_eq!(calendar_sentiment_score(None, Some(1.0)), 0.0);
        assert_eq!(calendar_sentiment_score(Some(1.0), None), 0.0);
    }

    #[test]
    fn maps_calendar_events_to_affected_assets() {
        assert_eq!(
            calendar_affected_assets("US", "CPI Inflation"),
            vec!["DXY", "US10Y"]
        );
        assert_eq!(
            calendar_affected_assets("United States", "Jobs Report"),
            vec!["DXY", "US10Y"]
        );
        assert_eq!(
            calendar_affected_assets("EU", "Oil inventories"),
            vec!["WTI"]
        );
        assert!(calendar_affected_assets("JP", "Consumer Confidence").is_empty());
    }

    #[test]
    fn calendar_hash_is_stable_sha256_prefix() {
        let event = EconomicCalendarItem {
            event: "CPI Inflation".to_string(),
            country: "US".to_string(),
            impact: "high".to_string(),
            unit: Some("%".to_string()),
            actual: None,
            estimate: Some(3.2),
            previous: Some(3.1),
            time: "2026-06-05 12:30:00".to_string(),
        };

        assert_eq!(calendar_hash(&event), "finnhub-economic-defcc2d276d6e318");
    }
}
