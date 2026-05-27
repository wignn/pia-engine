use std::time::Instant;

use chrono::{DateTime, Utc};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, USER_AGENT};
use rss::Channel;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use tokio::time::{self, Duration};
use tracing::{error, info, warn};

use crate::config::Config;

#[derive(Debug, sqlx::FromRow)]
struct NewsSource {
    id: String,
    name: String,
    rss_url: Option<String>,
    poll_interval_sec: i32,
}

struct ParsedArticle {
    title: String,
    url: String,
    summary: Option<String>,
    published_at: Option<DateTime<Utc>>,
    content_hash: String,
    sentiment: String,
    impact_level: String,
    currency_pairs: String,
}

pub async fn run(cfg: Config, pool: PgPool) {
    let client = match reqwest::Client::builder()
        .default_headers(default_headers())
        .timeout(Duration::from_secs(20))
        .build()
    {
        Ok(client) => client,
        Err(err) => {
            error!(error = %err, "failed to create news pipeline HTTP client");
            return;
        }
    };

    info!(
        rss_interval_sec = cfg.rss_fetch_sec,
        stock_interval_sec = cfg.stock_fetch_sec,
        "news ingestion pipeline running"
    );
    run_once(&pool, &client).await;

    let mut interval = time::interval(Duration::from_secs(cfg.rss_fetch_sec));
    loop {
        interval.tick().await;
        run_once(&pool, &client).await;
    }
}

async fn run_once(pool: &PgPool, client: &reqwest::Client) {
    let sources = match load_sources(pool).await {
        Ok(sources) => sources,
        Err(err) => {
            error!(error = %err, "failed to load news sources");
            return;
        }
    };

    for source in sources {
        if let Err(err) = poll_source(pool, client, &source).await {
            warn!(source = %source.name, error = %err, "news source poll failed");
        }
    }
}

async fn load_sources(pool: &PgPool) -> anyhow::Result<Vec<NewsSource>> {
    let sources = sqlx::query_as::<_, NewsSource>(
        "SELECT id, name, rss_url, category, poll_interval_sec FROM news.forex_news_sources WHERE is_active = TRUE AND source_type = 'rss' AND rss_url IS NOT NULL AND (next_allowed_poll_at IS NULL OR next_allowed_poll_at <= NOW()) ORDER BY priority ASC, name ASC",
    )
    .fetch_all(pool)
    .await?;

    Ok(sources)
}

async fn poll_source(
    pool: &PgPool,
    client: &reqwest::Client,
    source: &NewsSource,
) -> anyhow::Result<()> {
    let Some(url) = source.rss_url.as_deref() else {
        return Ok(());
    };

    let started = Instant::now();
    let response = client.get(url).send().await?;
    let status = response.status().as_u16() as i32;
    let body = response.text().await?;
    let latency_ms = started.elapsed().as_millis().min(i64::MAX as u128) as i64;

    if !(200..300).contains(&status) {
        record_source_error(pool, source, status, latency_ms, "non-success RSS response").await?;
        return Ok(());
    }

    let channel = Channel::read_from(body.as_bytes())?;
    let mut inserted = 0usize;
    for item in channel.items().iter().take(30) {
        let Some(title) = item
            .title()
            .map(str::trim)
            .filter(|title| !title.is_empty())
        else {
            continue;
        };
        let Some(url) = item
            .link()
            .or_else(|| item.guid().map(|guid| guid.value()))
            .map(str::trim)
            .filter(|url| !url.is_empty())
        else {
            continue;
        };

        let summary = item
            .description()
            .map(strip_html)
            .filter(|value| !value.is_empty());
        let published_at = item.pub_date().and_then(parse_rss_date);
        let article = ParsedArticle::new(title, url, summary, published_at);
        inserted += insert_forex_article(pool, source, &article).await?;
    }

    record_source_success(pool, source, status, latency_ms).await?;
    info!(source = %source.name, inserted, "news source polled");
    Ok(())
}

async fn insert_forex_article(
    pool: &PgPool,
    source: &NewsSource,
    article: &ParsedArticle,
) -> anyhow::Result<usize> {
    let inserted = sqlx::query(
        "WITH inserted AS (
            INSERT INTO news.forex_news_articles (source_id, content_hash, original_url, original_title, original_content, summary, is_processed, processed_at, published_at)
            VALUES ($1, $2, $3, $4, $5, $6, TRUE, NOW(), $7)
            ON CONFLICT (content_hash) DO NOTHING
            RETURNING id
        ), analysis AS (
            INSERT INTO news.forex_news_analyses (article_id, sentiment, impact_level, currency_pairs)
            SELECT id, $8, $9, $10 FROM inserted
            RETURNING article_id
        )
        SELECT COUNT(*)::BIGINT FROM inserted",
    )
    .bind(&source.id)
    .bind(&article.content_hash)
    .bind(&article.url)
    .bind(&article.title)
    .bind(article.summary.as_deref())
    .bind(article.summary.as_deref())
    .bind(article.published_at)
    .bind(&article.sentiment)
    .bind(&article.impact_level)
    .bind(&article.currency_pairs)
    .fetch_one(pool)
    .await?
    .try_get::<i64, _>(0)?;

    Ok(inserted as usize)
}

async fn record_source_success(
    pool: &PgPool,
    source: &NewsSource,
    status: i32,
    latency_ms: i64,
) -> anyhow::Result<()> {
    sqlx::query(
        "UPDATE news.forex_news_sources SET last_success_at = NOW(), last_status = $2, last_latency_ms = $3, success_count = success_count + 1, last_error_message = NULL, next_allowed_poll_at = NOW() + make_interval(secs => $4), updated_at = NOW() WHERE id = $1",
    )
    .bind(&source.id)
    .bind(status)
    .bind(latency_ms)
    .bind(source.poll_interval_sec.max(15))
    .execute(pool)
    .await?;

    Ok(())
}

async fn record_source_error(
    pool: &PgPool,
    source: &NewsSource,
    status: i32,
    latency_ms: i64,
    message: &str,
) -> anyhow::Result<()> {
    sqlx::query(
        "UPDATE news.forex_news_sources SET last_error_at = NOW(), last_status = $2, last_latency_ms = $3, error_count = error_count + 1, last_error_message = $4, next_allowed_poll_at = NOW() + make_interval(secs => $5), updated_at = NOW() WHERE id = $1",
    )
    .bind(&source.id)
    .bind(status)
    .bind(latency_ms)
    .bind(message)
    .bind(source.poll_interval_sec.max(60))
    .execute(pool)
    .await?;

    Ok(())
}

impl ParsedArticle {
    fn new(
        title: &str,
        url: &str,
        summary: Option<String>,
        published_at: Option<DateTime<Utc>>,
    ) -> Self {
        let body = summary.as_deref().unwrap_or_default();
        let text = format!("{title} {body}");
        let currency_pairs = detect_currency_pairs(&text).join(", ");
        let impact_level = detect_impact_level(&text).to_string();
        let sentiment = detect_sentiment(&text).to_string();

        Self {
            title: title.to_string(),
            url: url.to_string(),
            summary,
            published_at,
            content_hash: content_hash(url, title),
            sentiment,
            impact_level,
            currency_pairs,
        }
    }
}

fn content_hash(url: &str, title: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(url.trim().as_bytes());
    hasher.update(b"\0");
    hasher.update(title.trim().as_bytes());
    hex::encode(hasher.finalize())
}

fn parse_rss_date(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc2822(value)
        .or_else(|_| DateTime::parse_from_rfc3339(value))
        .map(|dt| dt.with_timezone(&Utc))
        .ok()
}

fn strip_html(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut in_tag = false;
    for ch in value.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => output.push(ch),
            _ => {}
        }
    }
    output
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn detect_currency_pairs(text: &str) -> Vec<&'static str> {
    const PAIRS: &[&str] = &[
        "EURUSD", "GBPUSD", "USDJPY", "AUDUSD", "USDCAD", "USDCHF", "NZDUSD", "EURJPY", "GBPJPY",
        "AUDJPY", "XAUUSD", "DXY",
    ];
    let normalized = text.to_uppercase().replace(['/', '-'], "");
    PAIRS
        .iter()
        .copied()
        .filter(|pair| normalized.contains(pair))
        .collect()
}

fn detect_impact_level(text: &str) -> &'static str {
    let lower = text.to_lowercase();
    if [
        "fed",
        "fomc",
        "ecb",
        "boj",
        "boe",
        "inflation",
        "cpi",
        "nfp",
        "payroll",
        "rate decision",
        "interest rate",
        "jobs report",
    ]
    .iter()
    .any(|term| lower.contains(term))
    {
        "high"
    } else if ["gdp", "pmi", "retail sales", "unemployment", "yield"]
        .iter()
        .any(|term| lower.contains(term))
    {
        "medium"
    } else {
        "low"
    }
}

fn detect_sentiment(text: &str) -> &'static str {
    let lower = text.to_lowercase();
    let positive = [
        "rally", "gain", "rise", "surge", "bull", "optimism", "rebound",
    ];
    let negative = [
        "fall", "drop", "slump", "bear", "risk-off", "concern", "weak", "decline",
    ];
    let pos = positive
        .iter()
        .filter(|term| lower.contains(**term))
        .count();
    let neg = negative
        .iter()
        .filter(|term| lower.contains(**term))
        .count();

    match pos.cmp(&neg) {
        std::cmp::Ordering::Greater => "positive",
        std::cmp::Ordering::Less => "negative",
        std::cmp::Ordering::Equal => "neutral",
    }
}

fn default_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("ATLSD-NewsService/1.0 (+https://wign.dev)"),
    );
    headers.insert(
        ACCEPT,
        HeaderValue::from_static("application/rss+xml, application/xml, text/xml, */*"),
    );
    headers
}

trait RowExt {
    fn try_get<T, I>(&self, index: I) -> Result<T, sqlx::Error>
    where
        T: for<'r> sqlx::Decode<'r, sqlx::Postgres> + sqlx::Type<sqlx::Postgres>,
        I: sqlx::ColumnIndex<sqlx::postgres::PgRow>;
}

impl RowExt for sqlx::postgres::PgRow {
    fn try_get<T, I>(&self, index: I) -> Result<T, sqlx::Error>
    where
        T: for<'r> sqlx::Decode<'r, sqlx::Postgres> + sqlx::Type<sqlx::Postgres>,
        I: sqlx::ColumnIndex<sqlx::postgres::PgRow>,
    {
        sqlx::Row::try_get(self, index)
    }
}
