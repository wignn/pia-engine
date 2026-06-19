use std::sync::RwLock;
use std::time::{Duration, Instant};

use axum::{
    extract::{Path, Query, State},
    Json,
};
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::{debug, error, warn};

use crate::state::AppState;

const FOREX_FACTORY_URL: &str = "https://nfs.faireconomy.media/ff_calendar_thisweek.json";
const CALENDAR_CACHE_TTL: Duration = Duration::from_secs(30 * 60);

static CALENDAR_CACHE: Lazy<RwLock<Option<(Value, Instant)>>> = Lazy::new(|| RwLock::new(None));
static CALENDAR_BACKOFF_UNTIL: Lazy<RwLock<Option<Instant>>> = Lazy::new(|| RwLock::new(None));

#[derive(Deserialize)]
pub struct LimitQuery {
    pub limit: Option<i64>,
}

#[derive(Deserialize)]
pub struct ForexNewsQuery {
    pub limit: Option<i64>,
    pub source: Option<String>,
    pub q: Option<String>,
}

#[derive(Deserialize)]
pub struct CalendarQuery {
    pub impact: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct FeedSourcePayload {
    pub name: String,
    pub url: String,
    pub rss_url: String,
    pub category: Option<String>,
    pub poll_interval_sec: Option<i32>,
    pub priority: Option<i32>,
    pub is_active: Option<bool>,
}

struct ValidFeedSourcePayload {
    name: String,
    slug: String,
    url: String,
    rss_url: String,
    category: String,
    poll_interval_sec: i32,
    priority: i32,
    is_active: bool,
}

type ForexNewsRow = (
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<chrono::DateTime<chrono::Utc>>,
    Option<chrono::DateTime<chrono::Utc>>,
);

#[derive(sqlx::FromRow)]
struct FeedSourceRow {
    id: String,
    name: String,
    slug: String,
    url: String,
    rss_url: Option<String>,
    category: String,
    poll_interval_sec: i32,
    priority: i32,
    is_active: bool,
    last_success_at: Option<chrono::DateTime<chrono::Utc>>,
    last_error_at: Option<chrono::DateTime<chrono::Utc>>,
    blocked_until: Option<chrono::DateTime<chrono::Utc>>,
    success_count: i64,
    error_count: i64,
    forbidden_count: i64,
    parse_error_count: i64,
    last_status: Option<i32>,
    last_latency_ms: Option<i64>,
}

pub async fn list_calendar(Query(query): Query<CalendarQuery>) -> Json<Value> {
    let impact_filter = query.impact.as_deref().unwrap_or("high").to_lowercase();
    let limit = query.limit.unwrap_or(10).clamp(1, 25);

    let (events, cache_status) = match get_calendar_events().await {
        Ok(result) => result,
        Err(error) => return Json(json!({ "error": error })),
    };

    let arr = match events.as_array() {
        Some(a) => a,
        None => return Json(json!({ "error": "unexpected calendar format" })),
    };

    let items: Vec<Value> = arr
        .iter()
        .filter(|ev| {
            let impact = ev["impact"].as_str().unwrap_or("").to_lowercase();
            match impact_filter.as_str() {
                "high" => impact.contains("high") || impact == "red",
                "medium" => impact.contains("medium") || impact == "orange",
                "low" => impact.contains("low") || impact == "yellow",
                _ => true,
            }
        })
        .take(limit)
        .map(|ev| {
            let country = ev["country"].as_str().unwrap_or("");
            json!({
                "title": ev["title"].as_str().unwrap_or(""),
                "currency": country,
                "date": ev["date"].as_str().unwrap_or(""),
                "impact": ev["impact"].as_str().unwrap_or(""),
                "forecast": ev["forecast"].as_str().unwrap_or(""),
                "previous": ev["previous"].as_str().unwrap_or(""),
                "actual": ev["actual"].as_str().unwrap_or(""),
            })
        })
        .collect();

    Json(json!({
        "items": items,
        "total": items.len(),
        "filter": { "impact": impact_filter, "limit": limit },
        "source": "forexfactory",
        "cache": { "status": cache_status },
    }))
}

pub async fn source_statuses(State(state): State<AppState>) -> Json<Value> {
    let rows = sqlx::query_as::<_, (String, String, Option<chrono::DateTime<chrono::Utc>>, Option<chrono::DateTime<chrono::Utc>>, Option<i32>, Option<i64>)>(
        "SELECT name, rss_url, last_success_at, last_error_at, last_status, last_latency_ms FROM news.forex_news_sources WHERE source_type = 'rss' ORDER BY priority ASC, name ASC",
    )
    .fetch_all(&state.db)
    .await;

    match rows {
        Ok(rows) => {
            let items: Vec<Value> = rows
                .into_iter()
                .map(|row| {
                    json!({
                        "name": row.0,
                        "rss_url": row.1,
                        "last_success_at": row.2,
                        "last_error_at": row.3,
                        "last_status": row.4,
                        "last_latency_ms": row.5,
                    })
                })
                .collect();
            Json(json!({ "items": items, "total": items.len() }))
        }
        Err(err) => {
            error!(error = %err, "forex source status query failed");
            Json(json!({ "error": "query failed" }))
        }
    }
}

pub async fn admin_list_forex_sources(State(state): State<AppState>) -> Json<Value> {
    let rows = sqlx::query_as::<_, FeedSourceRow>(
        "SELECT id, name, slug, url, rss_url, category, poll_interval_sec, priority, is_active, last_success_at, last_error_at, blocked_until, success_count, error_count, forbidden_count, parse_error_count, last_status, last_latency_ms FROM news.forex_news_sources WHERE source_type = 'rss' ORDER BY priority ASC, name ASC",
    )
    .fetch_all(&state.db)
    .await;

    match rows {
        Ok(rows) => {
            let items: Vec<Value> = rows.into_iter().map(feed_source_json).collect();
            Json(json!({ "items": items, "total": items.len() }))
        }
        Err(err) => {
            error!(error = %err, "admin forex source query failed");
            Json(json!({ "error": "query failed" }))
        }
    }
}

pub async fn admin_create_forex_source(
    State(state): State<AppState>,
    Json(payload): Json<FeedSourcePayload>,
) -> Json<Value> {
    let source = match validate_feed_source_payload(payload) {
        Ok(source) => source,
        Err(error) => return Json(json!({ "error": error })),
    };
    let id = format!("feed-{}", source.slug);

    let result = sqlx::query("INSERT INTO news.forex_news_sources (id, name, slug, source_type, url, rss_url, category, poll_interval_sec, priority, is_active, updated_at) VALUES ($1, $2, $3, 'rss', $4, $5, $6, $7, $8, $9, NOW())")
        .bind(&id)
        .bind(&source.name)
        .bind(&source.slug)
        .bind(&source.url)
        .bind(&source.rss_url)
        .bind(&source.category)
        .bind(source.poll_interval_sec)
        .bind(source.priority)
        .bind(source.is_active)
        .execute(&state.db)
        .await;

    match result {
        Ok(_) => Json(json!({ "id": id, "message": "Source created successfully" })),
        Err(err) => {
            warn!(error = %err, id = %id, "admin forex source create failed");
            Json(json!({ "error": "source create failed" }))
        }
    }
}

pub async fn admin_update_forex_source(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<FeedSourcePayload>,
) -> Json<Value> {
    let source = match validate_feed_source_payload(payload) {
        Ok(source) => source,
        Err(error) => return Json(json!({ "error": error })),
    };

    let result = sqlx::query("UPDATE news.forex_news_sources SET name = $2, slug = $3, url = $4, rss_url = $5, category = $6, poll_interval_sec = $7, priority = $8, is_active = $9, updated_at = NOW() WHERE id = $1")
        .bind(&id)
        .bind(&source.name)
        .bind(&source.slug)
        .bind(&source.url)
        .bind(&source.rss_url)
        .bind(&source.category)
        .bind(source.poll_interval_sec)
        .bind(source.priority)
        .bind(source.is_active)
        .execute(&state.db)
        .await;

    match result {
        Ok(result) if result.rows_affected() == 0 => Json(json!({ "error": "source not found" })),
        Ok(_) => Json(json!({ "message": "Source updated successfully" })),
        Err(err) => {
            warn!(error = %err, id = %id, "admin forex source update failed");
            Json(json!({ "error": "source update failed" }))
        }
    }
}

pub async fn admin_toggle_forex_source(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<Value> {
    let row = sqlx::query_as::<_, (bool,)>(
        "UPDATE news.forex_news_sources SET is_active = NOT is_active, updated_at = NOW() WHERE id = $1 RETURNING is_active",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await;

    match row {
        Ok(Some((is_active,))) => Json(json!({
            "message": "Source status updated",
            "is_active": is_active,
        })),
        Ok(None) => Json(json!({ "error": "source not found" })),
        Err(err) => {
            warn!(error = %err, id = %id, "admin forex source toggle failed");
            Json(json!({ "error": "source toggle failed" }))
        }
    }
}

pub async fn admin_test_forex_source(Json(payload): Json<FeedSourcePayload>) -> Json<Value> {
    let source = match validate_feed_source_payload(payload) {
        Ok(source) => source,
        Err(error) => return Json(json!({ "ok": false, "error": error })),
    };

    let started = Instant::now();
    match test_rss_source(&source.rss_url).await {
        Ok(entries) => Json(json!({
            "ok": true,
            "entries": entries,
            "latency_ms": started.elapsed().as_millis().min(u64::MAX as u128) as u64,
        })),
        Err(error) => Json(json!({ "ok": false, "error": error })),
    }
}

pub async fn list_forex_news(
    State(state): State<AppState>,
    Query(query): Query<ForexNewsQuery>,
) -> Json<Value> {
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let search = query.q.unwrap_or_default();
    let source = query.source.unwrap_or_default();

    let rows = sqlx::query_as::<_, ForexNewsRow>(
        "SELECT a.id::text, a.original_title, a.summary, COALESCE(s.name, 'Unknown') AS source_name, a.original_url, an.sentiment, an.impact_level, a.published_at, a.processed_at FROM news.forex_news_articles a LEFT JOIN news.forex_news_sources s ON a.source_id = s.id LEFT JOIN news.forex_news_analyses an ON a.id = an.article_id WHERE a.is_processed = TRUE AND ($2 = '' OR COALESCE(s.name, '') ILIKE '%' || $2 || '%') AND ($3 = '' OR a.original_title ILIKE '%' || $3 || '%' OR COALESCE(a.summary, '') ILIKE '%' || $3 || '%') ORDER BY COALESCE(a.processed_at, a.published_at, a.created_at) DESC LIMIT $1",
    )
    .bind(limit)
    .bind(source)
    .bind(search)
    .fetch_all(&state.db)
    .await;

    forex_rows_response(rows)
}

pub async fn latest_forex_news(
    State(state): State<AppState>,
    Query(query): Query<LimitQuery>,
) -> Json<Value> {
    let limit = query.limit.unwrap_or(10).clamp(1, 50);
    let rows = sqlx::query_as::<_, ForexNewsRow>(
        "SELECT a.id::text, a.original_title, a.summary, COALESCE(s.name, 'Unknown') AS source_name, a.original_url, an.sentiment, an.impact_level, a.published_at, a.processed_at FROM news.forex_news_articles a LEFT JOIN news.forex_news_sources s ON a.source_id = s.id LEFT JOIN news.forex_news_analyses an ON a.id = an.article_id WHERE a.is_processed = TRUE ORDER BY COALESCE(a.processed_at, a.published_at, a.created_at) DESC LIMIT $1",
    )
    .bind(limit)
    .fetch_all(&state.db)
    .await;

    forex_rows_response(rows)
}

pub async fn get_forex_news(State(state): State<AppState>, Path(id): Path<String>) -> Json<Value> {
    let row = sqlx::query_as::<_, (String, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<chrono::DateTime<chrono::Utc>>, Option<chrono::DateTime<chrono::Utc>>)>(
        "SELECT a.id::text, a.original_title, a.summary, COALESCE(s.name, 'Unknown') AS source_name, a.original_url, an.sentiment, an.impact_level, a.published_at, a.processed_at FROM news.forex_news_articles a LEFT JOIN news.forex_news_sources s ON a.source_id = s.id LEFT JOIN news.forex_news_analyses an ON a.id = an.article_id WHERE a.id::text = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await;

    match row {
        Ok(Some(row)) => Json(forex_row_json(row)),
        Ok(None) => Json(json!({ "error": "news not found" })),
        Err(err) => {
            error!(error = %err, "forex news query failed");
            Json(json!({ "error": "query failed" }))
        }
    }
}

pub async fn latest_stock_news(
    State(state): State<AppState>,
    Query(query): Query<LimitQuery>,
) -> Json<Value> {
    let limit = query.limit.unwrap_or(10).clamp(1, 50);
    let rows = sqlx::query_as::<_, (String, String, Option<String>, String, String, Option<String>, Option<String>, Option<String>, Option<chrono::DateTime<chrono::Utc>>)>(
        "SELECT content_hash, title, summary, source_name, category, tickers, sentiment, impact_level, processed_at FROM news.stock_news WHERE is_processed = TRUE ORDER BY processed_at DESC LIMIT $1",
    )
    .bind(limit)
    .fetch_all(&state.db)
    .await;

    match rows {
        Ok(rows) => {
            let items: Vec<Value> = rows
                .into_iter()
                .map(|row| {
                    json!({
                        "id": row.0,
                        "content_hash": row.0,
                        "title": row.1,
                        "summary": row.2,
                        "source_name": row.3,
                        "category": row.4,
                        "tickers": row.5,
                        "sentiment": row.6,
                        "impact_level": row.7,
                        "published_at": row.8,
                        "processed_at": row.8,
                    })
                })
                .collect();
            Json(json!({ "items": items, "total": items.len() }))
        }
        Err(err) => {
            error!(error = %err, "stock news query failed");
            Json(json!({ "error": "query failed" }))
        }
    }
}

pub async fn macro_dashboard(
    State(state): State<AppState>,
    Query(query): Query<LimitQuery>,
) -> Json<Value> {
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    match crate::pipeline::r#macro::dashboard(&state.db, limit).await {
        Ok(response) => Json(response),
        Err(err) => {
            error!(error = %err, "macro dashboard query failed");
            Json(json!({ "error": "query failed" }))
        }
    }
}

fn forex_rows_response(rows: Result<Vec<ForexNewsRow>, sqlx::Error>) -> Json<Value> {
    match rows {
        Ok(rows) => {
            let items: Vec<Value> = rows.into_iter().map(forex_row_json).collect();
            Json(json!({ "items": items, "total": items.len() }))
        }
        Err(err) => {
            error!(error = %err, "forex news query failed");
            Json(json!({ "error": "query failed" }))
        }
    }
}

fn forex_row_json(row: ForexNewsRow) -> Value {
    json!({
        "id": row.0,
        "title": row.1,
        "original_title": row.1,
        "summary": row.2,
        "source_name": row.3,
        "url": row.4,
        "original_url": row.4,
        "sentiment": row.5,
        "impact_level": row.6,
        "published_at": row.7,
        "processed_at": row.8,
    })
}

fn feed_source_json(row: FeedSourceRow) -> Value {
    json!({
        "id": row.id,
        "name": row.name,
        "slug": row.slug,
        "url": row.url,
        "rss_url": row.rss_url,
        "category": row.category,
        "poll_interval_sec": row.poll_interval_sec,
        "priority": row.priority,
        "is_active": row.is_active,
        "last_success_at": row.last_success_at,
        "last_error_at": row.last_error_at,
        "blocked_until": row.blocked_until,
        "success_count": row.success_count,
        "error_count": row.error_count,
        "forbidden_count": row.forbidden_count,
        "parse_error_count": row.parse_error_count,
        "last_status": row.last_status,
        "last_latency_ms": row.last_latency_ms,
    })
}

fn validate_feed_source_payload(
    payload: FeedSourcePayload,
) -> Result<ValidFeedSourcePayload, &'static str> {
    let name = payload.name.trim();
    if name.is_empty() {
        return Err("name is required");
    }
    let url = payload.url.trim();
    let rss_url = payload.rss_url.trim();
    if !is_http_url(url) {
        return Err("url must be http or https");
    }
    if !is_http_url(rss_url) {
        return Err("rss_url must be http or https");
    }
    let poll_interval_sec = payload.poll_interval_sec.unwrap_or(45);
    if poll_interval_sec < 15 {
        return Err("poll_interval_sec must be at least 15");
    }
    let priority = payload.priority.unwrap_or(100).max(0);
    let category = payload
        .category
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("forex")
        .to_string();

    Ok(ValidFeedSourcePayload {
        name: name.to_string(),
        slug: slugify(name),
        url: url.to_string(),
        rss_url: rss_url.to_string(),
        category,
        poll_interval_sec,
        priority,
        is_active: payload.is_active.unwrap_or(true),
    })
}

fn is_http_url(value: &str) -> bool {
    value.starts_with("https://") || value.starts_with("http://")
}

fn slugify(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

async fn test_rss_source(rss_url: &str) -> Result<usize, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent("ATLSD feed-source-test/1.0")
        .build()
        .map_err(|error| format!("internal client error: {error}"))?;
    let response = client
        .get(rss_url)
        .send()
        .await
        .map_err(|error| format!("upstream request failed: {error}"))?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("upstream returned status: {status}"));
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|error| format!("upstream body error: {error}"))?;
    let channel =
        rss::Channel::read_from(&bytes[..]).map_err(|error| format!("rss parse error: {error}"))?;
    Ok(channel.items().len())
}

async fn get_calendar_events() -> Result<(Value, &'static str), String> {
    if let Ok(guard) = CALENDAR_CACHE.read() {
        if let Some((events, cached_at)) = guard.as_ref() {
            if cached_at.elapsed() < CALENDAR_CACHE_TTL {
                debug!("using cached calendar response");
                return Ok((events.clone(), "hit"));
            }
        }
    }

    if let Ok(guard) = CALENDAR_BACKOFF_UNTIL.read() {
        if let Some(until) = *guard {
            if Instant::now() < until {
                if let Some(events) = stale_calendar_events() {
                    return Ok((events, "stale"));
                }
            }
        }
    }

    match fetch_calendar_events().await {
        Ok(events) => {
            if let Ok(mut guard) = CALENDAR_CACHE.write() {
                *guard = Some((events.clone(), Instant::now()));
            }
            if let Ok(mut guard) = CALENDAR_BACKOFF_UNTIL.write() {
                *guard = None;
            }
            Ok((events, "miss"))
        }
        Err(error) => {
            if let Ok(mut guard) = CALENDAR_BACKOFF_UNTIL.write() {
                *guard = Some(Instant::now() + Duration::from_secs(15 * 60));
            }
            if let Some(events) = stale_calendar_events() {
                warn!(error = %error, "returning stale forex factory calendar cache");
                Ok((events, "stale"))
            } else {
                Err(error)
            }
        }
    }
}

async fn fetch_calendar_events() -> Result<Value, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .map_err(|error| format!("internal client error: {error}"))?;

    let response = client
        .get(FOREX_FACTORY_URL)
        .send()
        .await
        .map_err(|error| format!("upstream request failed: {error}"))?;

    let status = response.status();
    if !status.is_success() {
        warn!(status = %status, "forex factory calendar request returned non-success status");
        return Err(format!("upstream returned status: {status}"));
    }

    response
        .json::<Value>()
        .await
        .map_err(|error| format!("upstream parse error: {error}"))
}

fn stale_calendar_events() -> Option<Value> {
    CALENDAR_CACHE
        .read()
        .ok()
        .and_then(|guard| guard.as_ref().map(|(events, _)| events.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feed_source_payload_defaults_match_admin_form_contract() {
        let payload = FeedSourcePayload {
            name: "FXStreet".to_string(),
            url: "https://www.fxstreet.com".to_string(),
            rss_url: "https://www.fxstreet.com/rss/news".to_string(),
            category: None,
            poll_interval_sec: None,
            priority: None,
            is_active: None,
        };

        let normalized = validate_feed_source_payload(payload).unwrap();

        assert_eq!(normalized.slug, "fxstreet");
        assert_eq!(normalized.category, "forex");
        assert_eq!(normalized.poll_interval_sec, 45);
        assert_eq!(normalized.priority, 100);
        assert!(normalized.is_active);
    }

    #[test]
    fn feed_source_payload_rejects_invalid_urls_and_short_polling() {
        let payload = FeedSourcePayload {
            name: "Bad".to_string(),
            url: "not-a-url".to_string(),
            rss_url: "https://example.com/feed.xml".to_string(),
            category: Some("forex".to_string()),
            poll_interval_sec: Some(10),
            priority: Some(100),
            is_active: Some(true),
        };

        assert!(validate_feed_source_payload(payload).is_err());
    }
}
