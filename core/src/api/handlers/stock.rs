use axum::{extract::{Query, State}, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::error;

use crate::api::state::AppState;

#[derive(Deserialize)]
pub struct LatestQuery {
    pub limit: Option<i64>,
}

pub async fn latest_stock_news(
    State(state): State<AppState>,
    Query(query): Query<LatestQuery>,
) -> Json<Value> {
    let limit = query.limit.unwrap_or(10).clamp(1, 50);

    let rows = sqlx::query_as::<_, (String, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<chrono::DateTime<chrono::Utc>>)>(
        "SELECT content_hash, title, summary, source_name, \
         category, tickers, sentiment, impact_level, processed_at \
         FROM stock_news \
         WHERE is_processed = TRUE \
         ORDER BY processed_at DESC \
         LIMIT $1"
    )
    .bind(limit)
    .fetch_all(&state.db)
    .await;

    let items: Vec<Value> = match rows {
        Ok(rows) => rows.iter().map(|r| json!({
            "id": r.0,
            "content_hash": r.0,
            "title": r.1,
            "summary": r.2,
            "source_name": r.3,
            "category": r.4,
            "tickers": r.5,
            "sentiment": r.6,
            "impact_level": r.7,
            "published_at": r.8,
            "processed_at": r.8,
        })).collect(),
        Err(e) => {
            error!(error = %e, "stock news query failed");
            return Json(json!({ "error": "query failed" }));
        }
    };

    Json(json!({
        "items": items,
        "total": items.len(),
    }))
}
