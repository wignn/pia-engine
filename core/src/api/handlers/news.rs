use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::error;

use crate::api::state::AppState;

#[derive(Deserialize)]
pub struct ListQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

pub async fn list_news(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Json<Value> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * page_size;

    let rows = sqlx::query_as::<_, (String, Option<String>, String, String, String, Option<String>, bool, Option<chrono::DateTime<chrono::Utc>>, Option<chrono::DateTime<chrono::Utc>>)>(
        "SELECT id::text, source_id, content_hash, original_url, original_title, \
         summary, is_processed, processed_at, published_at \
         FROM news_articles \
         ORDER BY processed_at DESC NULLS LAST \
         LIMIT $1 OFFSET $2"
    )
    .bind(page_size)
    .bind(offset)
    .fetch_all(&state.db)
    .await;

    let items: Vec<Value> = match rows {
        Ok(rows) => rows.iter().map(|r| json!({
            "id": r.0,
            "content_hash": r.2,
            "original_url": r.3,
            "original_title": r.4,
            "summary": r.5,
            "is_processed": r.6,
            "processed_at": r.7,
            "published_at": r.8,
        })).collect(),
        Err(e) => {
            error!(error = %e, "list news query failed");
            return Json(json!({ "error": "query failed" }));
        }
    };

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM news_articles")
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

pub async fn latest_news(
    State(state): State<AppState>,
    Query(query): Query<LatestQuery>,
) -> Json<Value> {
    let limit = query.limit.unwrap_or(10).clamp(1, 50);

    let rows = sqlx::query_as::<_, (String, String, String, String, Option<String>, Option<String>, Option<chrono::DateTime<chrono::Utc>>, Option<chrono::DateTime<chrono::Utc>>, String, Option<String>, Option<String>, Option<String>)>(
        "SELECT a.id::text, a.content_hash, a.original_url, a.original_title, \
         a.translated_title, a.summary, a.published_at, a.processed_at, \
         COALESCE(s.name, 'Unknown') AS source_name, \
         an.sentiment, an.impact_level, an.currency_pairs \
         FROM news_articles a \
         LEFT JOIN news_sources s ON a.source_id = s.id \
         LEFT JOIN news_analyses an ON an.article_id = a.id \
         WHERE a.is_processed = TRUE \
         ORDER BY a.processed_at DESC NULLS LAST \
         LIMIT $1"
    )
    .bind(limit)
    .fetch_all(&state.db)
    .await;

    let items: Vec<Value> = match rows {
        Ok(rows) => rows.iter().map(|r| json!({
            "id": r.0,
            "content_hash": r.1,
            "original_url": r.2,
            "original_title": r.3,
            "translated_title": r.4,
            "summary": r.5,
            "published_at": r.6,
            "processed_at": r.7,
            "source_name": r.8,
            "sentiment": r.9,
            "impact_level": r.10,
            "currency_pairs": r.11,
        })).collect(),
        Err(e) => {
            error!(error = %e, "latest news query failed");
            return Json(json!({ "error": "query failed" }));
        }
    };

    Json(json!({
        "items": items,
        "total": items.len(),
    }))
}

pub async fn get_news(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<Value> {
    let row = sqlx::query_as::<_, (String, Option<String>, String, String, String, Option<String>, Option<String>, bool, Option<chrono::DateTime<chrono::Utc>>, Option<chrono::DateTime<chrono::Utc>>)>(
        "SELECT id::text, source_id, content_hash, original_url, original_title, \
         original_content, summary, is_processed, processed_at, published_at \
         FROM news_articles WHERE id::text = $1"
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
            "original_title": r.4,
            "original_content": r.5,
            "summary": r.6,
            "is_processed": r.7,
            "processed_at": r.8,
            "published_at": r.9,
        })),
        Ok(None) => Json(json!({ "error": "article not found" })),
        Err(e) => {
            error!(error = %e, "get news query failed");
            Json(json!({ "error": "query failed" }))
        }
    }
}
