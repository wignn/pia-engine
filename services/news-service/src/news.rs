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
