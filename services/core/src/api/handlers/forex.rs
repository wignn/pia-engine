use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::error;

use crate::api::state::AppState;
use crate::collector::forex::{default_forex_feeds, FeedSource};
use crate::tenant::context::TenantContext;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ManagedFeedSource {
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
    consecutive_403: i32,
    success_count: i64,
    error_count: i64,
    forbidden_count: i64,
    parse_error_count: i64,
    last_status: Option<i32>,
    last_latency_ms: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct FeedSourcePayload {
    name: String,
    url: String,
    rss_url: String,
    category: Option<String>,
    poll_interval_sec: Option<i32>,
    priority: Option<i32>,
    is_active: Option<bool>,
}

fn require_admin(ctx: &TenantContext) -> Result<(), StatusCode> {
    if ctx.is_admin {
        Ok(())
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

fn validate_feed_payload(payload: &FeedSourcePayload) -> Result<(), &'static str> {
    if payload.name.trim().is_empty()
        || payload.url.trim().is_empty()
        || payload.rss_url.trim().is_empty()
    {
        return Err("name, url, and rss_url are required");
    }
    if !(payload.url.starts_with("https://") || payload.url.starts_with("http://")) {
        return Err("url must start with http:// or https://");
    }
    if !(payload.rss_url.starts_with("https://") || payload.rss_url.starts_with("http://")) {
        return Err("rss_url must start with http:// or https://");
    }
    Ok(())
}

fn to_slug(input: &str) -> String {
    input
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

async fn load_managed_feed_sources(db: &sqlx::PgPool) -> Result<Vec<FeedSource>, sqlx::Error> {
    let rows: Vec<(String, String, String, String, String, i32)> = sqlx::query_as(
        "SELECT id, name, url, rss_url, category, poll_interval_sec \
         FROM news.forex_news_sources \
         WHERE is_active = TRUE AND source_type = 'rss' AND rss_url IS NOT NULL \
         ORDER BY priority ASC, name ASC",
    )
    .fetch_all(db)
    .await?;

    Ok(rows
        .into_iter()
        .map(
            |(id, name, url, rss_url, category, poll_interval_sec)| FeedSource {
                id: Some(id),
                name,
                url,
                rss_url,
                category,
                poll_interval_sec: Some(poll_interval_sec.max(15) as u64),
            },
        )
        .collect())
}

async fn list_managed_sources(db: &sqlx::PgPool) -> Result<Vec<ManagedFeedSource>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, name, slug, url, rss_url, category, poll_interval_sec, priority, is_active, \
         last_success_at, last_error_at, blocked_until, consecutive_403, success_count, error_count, forbidden_count, \
         parse_error_count, last_status, last_latency_ms \
         FROM news.forex_news_sources WHERE source_type = 'rss' ORDER BY priority ASC, name ASC",
    )
    .fetch_all(db)
    .await
}

pub async fn admin_list_sources(
    State(state): State<AppState>,
    Extension(ctx): Extension<TenantContext>,
) -> Result<Json<Value>, StatusCode> {
    require_admin(&ctx)?;
    match list_managed_sources(&state.db).await {
        Ok(sources) => Ok(Json(json!({ "items": sources, "total": sources.len() }))),
        Err(e) => {
            error!(error = %e, "list feed sources query failed");
            Ok(Json(json!({ "error": "query failed" })))
        }
    }
}

pub async fn admin_create_source(
    State(state): State<AppState>,
    Extension(ctx): Extension<TenantContext>,
    Json(payload): Json<FeedSourcePayload>,
) -> Result<Json<Value>, StatusCode> {
    require_admin(&ctx)?;
    if let Err(message) = validate_feed_payload(&payload) {
        return Ok(Json(json!({ "error": message })));
    }
    let slug = to_slug(&payload.name);
    let id = format!("feed-{slug}");
    let category = payload.category.unwrap_or_else(|| "forex".to_string());
    let poll_interval_sec = payload.poll_interval_sec.unwrap_or(45).max(15);
    let priority = payload.priority.unwrap_or(100);
    let is_active = payload.is_active.unwrap_or(true);

    let res = sqlx::query(
        "INSERT INTO news.forex_news_sources (id, name, slug, source_type, url, rss_url, category, poll_interval_sec, priority, is_active, updated_at) \
         VALUES ($1, $2, $3, 'rss', $4, $5, $6, $7, $8, $9, NOW())",
    )
    .bind(&id)
    .bind(payload.name.trim())
    .bind(&slug)
    .bind(payload.url.trim())
    .bind(payload.rss_url.trim())
    .bind(&category)
    .bind(poll_interval_sec)
    .bind(priority)
    .bind(is_active)
    .execute(&state.db)
    .await;

    match res {
        Ok(_) => Ok(Json(json!({ "id": id, "message": "source created" }))),
        Err(e) => {
            error!(error = %e, "create feed source failed");
            Ok(Json(json!({ "error": "create failed" })))
        }
    }
}

pub async fn admin_update_source(
    State(state): State<AppState>,
    Extension(ctx): Extension<TenantContext>,
    Path(id): Path<String>,
    Json(payload): Json<FeedSourcePayload>,
) -> Result<Json<Value>, StatusCode> {
    require_admin(&ctx)?;
    if let Err(message) = validate_feed_payload(&payload) {
        return Ok(Json(json!({ "error": message })));
    }
    let category = payload.category.unwrap_or_else(|| "forex".to_string());
    let poll_interval_sec = payload.poll_interval_sec.unwrap_or(45).max(15);
    let priority = payload.priority.unwrap_or(100);
    let is_active = payload.is_active.unwrap_or(true);

    let res = sqlx::query(
        "UPDATE news.forex_news_sources SET name = $2, url = $3, rss_url = $4, category = $5, \
         poll_interval_sec = $6, priority = $7, is_active = $8, updated_at = NOW() WHERE id = $1",
    )
    .bind(&id)
    .bind(payload.name.trim())
    .bind(payload.url.trim())
    .bind(payload.rss_url.trim())
    .bind(&category)
    .bind(poll_interval_sec)
    .bind(priority)
    .bind(is_active)
    .execute(&state.db)
    .await;

    match res {
        Ok(result) if result.rows_affected() > 0 => {
            Ok(Json(json!({ "message": "source updated" })))
        }
        Ok(_) => Ok(Json(json!({ "error": "source not found" }))),
        Err(e) => {
            error!(error = %e, id = %id, "update feed source failed");
            Ok(Json(json!({ "error": "update failed" })))
        }
    }
}

pub async fn admin_toggle_source(
    State(state): State<AppState>,
    Extension(ctx): Extension<TenantContext>,
    Path(id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    require_admin(&ctx)?;
    let res = sqlx::query(
        "UPDATE news.forex_news_sources SET is_active = NOT is_active, updated_at = NOW() WHERE id = $1 RETURNING is_active",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await;

    match res {
        Ok(Some(row)) => {
            let is_active: bool = row.get("is_active");
            Ok(Json(
                json!({ "message": "source toggled", "is_active": is_active }),
            ))
        }
        Ok(None) => Ok(Json(json!({ "error": "source not found" }))),
        Err(e) => {
            error!(error = %e, id = %id, "toggle feed source failed");
            Ok(Json(json!({ "error": "toggle failed" })))
        }
    }
}

pub async fn admin_test_source(
    State(state): State<AppState>,
    Extension(ctx): Extension<TenantContext>,
    Json(payload): Json<FeedSourcePayload>,
) -> Result<Json<Value>, StatusCode> {
    require_admin(&ctx)?;
    if let Err(message) = validate_feed_payload(&payload) {
        return Ok(Json(json!({ "ok": false, "error": message })));
    }
    let source = FeedSource {
        id: None,
        name: payload.name,
        url: payload.url,
        rss_url: payload.rss_url,
        category: payload.category.unwrap_or_else(|| "forex".to_string()),
        poll_interval_sec: payload.poll_interval_sec.map(|value| value.max(15) as u64),
    };
    let started = std::time::Instant::now();
    let entries = state.forex_collector.fetch_feed(&source).await;
    Ok(Json(json!({
        "ok": !entries.is_empty(),
        "entries": entries.len(),
        "latency_ms": started.elapsed().as_millis(),
    })))
}

use sqlx::Row;

pub async fn source_statuses(State(state): State<AppState>) -> Json<Value> {
    let feeds = match load_managed_feed_sources(&state.db).await {
        Ok(feeds) if !feeds.is_empty() => feeds,
        Ok(_) | Err(_) => default_forex_feeds(),
    };
    let mut items = state.forex_collector.source_statuses(&feeds).await;

    if let Ok(managed) = list_managed_sources(&state.db).await {
        for item in &mut items {
            if item.success_count > 0 || item.error_count > 0 {
                continue;
            }
            let Some(source) = managed
                .iter()
                .find(|source| source.rss_url.as_deref() == Some(item.rss_url.as_str()))
            else {
                continue;
            };
            item.last_success_at = source.last_success_at;
            item.last_error_at = source.last_error_at;
            item.blocked_until = source.blocked_until;
            item.consecutive_403 = source.consecutive_403.max(0) as u32;
            item.success_count = source.success_count.max(0) as u64;
            item.error_count = source.error_count.max(0) as u64;
            item.forbidden_count = source.forbidden_count.max(0) as u64;
            item.parse_error_count = source.parse_error_count.max(0) as u64;
            item.last_status = source.last_status.map(|status| status as u16);
            item.last_latency_ms = source.last_latency_ms.map(|latency| latency.max(0) as u128);
            item.status = if source
                .blocked_until
                .is_some_and(|blocked_until| blocked_until > chrono::Utc::now())
            {
                "blocked".to_string()
            } else if source.last_error_at.is_some()
                && source.last_error_at > source.last_success_at
            {
                "error".to_string()
            } else if source.last_success_at.is_some() {
                "ok".to_string()
            } else {
                item.status.clone()
            };
        }
    }

    Json(json!({
        "items": items,
        "total": items.len(),
    }))
}

#[derive(Deserialize)]
pub struct ListQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

pub async fn list_forex_news(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Json<Value> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * page_size;

    let rows = sqlx::query_as::<
        _,
        (
            String,
            Option<String>,
            String,
            String,
            String,
            Option<String>,
            bool,
            Option<chrono::DateTime<chrono::Utc>>,
            Option<chrono::DateTime<chrono::Utc>>,
            Option<String>,
            Option<String>,
        ),
    >(
        "SELECT a.id::text, a.source_id, a.content_hash, a.original_url, a.original_title, \
         a.summary, a.is_processed, a.processed_at, a.published_at, \
         an.sentiment, an.impact_level \
         FROM news.forex_news_articles a \
         LEFT JOIN news.forex_news_analyses an ON a.id = an.article_id \
         ORDER BY a.processed_at DESC NULLS LAST \
         LIMIT $1 OFFSET $2",
    )
    .bind(page_size)
    .bind(offset)
    .fetch_all(&state.db)
    .await;

    let items: Vec<Value> = match rows {
        Ok(rows) => rows
            .iter()
            .map(|r| {
                json!({
                    "id": r.0,
                    "content_hash": r.2,
                    "original_url": r.3,
                    "title": r.4,
                    "original_title": r.4,
                    "summary": r.5,
                    "is_processed": r.6,
                    "processed_at": r.7,
                    "published_at": r.8,
                    "sentiment": r.9,
                    "impact_level": r.10,
                })
            })
            .collect(),
        Err(e) => {
            error!(error = %e, "list forex news query failed");
            return Json(json!({ "error": "query failed" }));
        }
    };

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM news.forex_news_articles")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    let total_pages = (total + page_size - 1) / page_size;

    Json(json!({
        "items": items,
        "total": total,
        "page": page,
        "page_size": page_size,
        "total_pages": total_pages,
    }))
}

#[derive(Deserialize)]
pub struct LatestQuery {
    pub limit: Option<i64>,
}

pub async fn latest_forex_news(
    State(state): State<AppState>,
    Query(query): Query<LatestQuery>,
) -> Json<Value> {
    let limit = query.limit.unwrap_or(10).clamp(1, 50);

    let rows = sqlx::query_as::<
        _,
        (
            String,
            String,
            String,
            String,
            Option<String>,
            Option<String>,
            Option<chrono::DateTime<chrono::Utc>>,
            Option<chrono::DateTime<chrono::Utc>>,
            String,
            Option<String>,
            Option<String>,
        ),
    >(
        "SELECT a.id::text, a.content_hash, a.original_url, a.original_title, \
         a.translated_title, a.summary, a.published_at, a.processed_at, \
         COALESCE(s.name, 'Unknown') AS source_name, \
         an.sentiment, an.impact_level \
         FROM news.forex_news_articles a \
         LEFT JOIN news.forex_news_sources s ON a.source_id = s.id \
         LEFT JOIN news.forex_news_analyses an ON a.id = an.article_id \
         WHERE a.is_processed = TRUE \
         ORDER BY a.processed_at DESC NULLS LAST \
         LIMIT $1",
    )
    .bind(limit)
    .fetch_all(&state.db)
    .await;

    let items: Vec<Value> = match rows {
        Ok(rows) => rows
            .iter()
            .map(|r| {
                json!({
                    "id": r.0,
                    "content_hash": r.1,
                    "original_url": r.2,
                    "title": r.3,
                    "original_title": r.3,
                    "translated_title": r.4,
                    "summary": r.5,
                    "published_at": r.6,
                    "processed_at": r.7,
                    "source_name": r.8,
                    "sentiment": r.9,
                    "impact_level": r.10,
                    "currency_pairs": null,
                })
            })
            .collect(),
        Err(e) => {
            error!(error = %e, "latest forex news query failed");
            return Json(json!({ "error": "query failed" }));
        }
    };

    Json(json!({
        "items": items,
        "total": items.len(),
    }))
}

pub async fn get_forex_news(State(state): State<AppState>, Path(id): Path<String>) -> Json<Value> {
    let row = sqlx::query_as::<
        _,
        (
            String,
            Option<String>,
            String,
            String,
            String,
            Option<String>,
            Option<String>,
            bool,
            Option<chrono::DateTime<chrono::Utc>>,
            Option<chrono::DateTime<chrono::Utc>>,
        ),
    >(
        "SELECT id::text, source_id, content_hash, original_url, original_title, \
         original_content, summary, is_processed, processed_at, published_at \
         FROM news.forex_news_articles WHERE id::text = $1",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await;

    match row {
        Ok(Some(r)) => Json(json!({
            "id": r.0,
            "source_id": r.1,
            "content_hash": r.2,
            "original_url": r.3,
            "title": r.4,
            "original_title": r.4,
            "original_content": r.5,
            "summary": r.6,
            "is_processed": r.7,
            "processed_at": r.8,
            "published_at": r.9,
        })),
        Ok(None) => Json(json!({ "error": "article not found" })),
        Err(e) => {
            error!(error = %e, "get forex news query failed");
            Json(json!({ "error": "query failed" }))
        }
    }
}
