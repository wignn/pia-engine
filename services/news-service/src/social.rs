use axum::{
    extract::{Query, State},
    Json,
};
use chrono::{DateTime, Utc};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{types::Json as SqlxJson, FromRow};
use tracing::{info, warn};

use crate::state::AppState;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct SocialPost {
    pub event_id: String,
    pub post_id: String,
    pub platform: String,
    pub source_account: String,
    pub author_username: String,
    pub author_display_name: String,
    pub text: String,
    pub url: String,
    pub created_at: DateTime<Utc>,
    pub fetched_at: DateTime<Utc>,
    pub reply_count: i64,
    pub retweet_count: i64,
    pub like_count: i64,
    pub quote_count: i64,
    pub language: String,
    pub media_urls: Value,
}

#[derive(Debug, Deserialize)]
pub struct SocialQuery {
    pub platform: Option<String>,
    pub account: Option<String>,
    pub q: Option<String>,
    pub before: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
}

pub async fn list_posts(
    State(state): State<AppState>,
    Query(query): Query<SocialQuery>,
) -> Json<Value> {
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let rows = sqlx::query_as::<_, SocialPost>(
        "SELECT event_id, post_id, platform, source_account, author_username,
                author_display_name, text, url, created_at, fetched_at,
                reply_count, retweet_count, like_count, quote_count,
                language, media_urls
         FROM news.social_posts
         WHERE ($1::text IS NULL OR platform = $1)
           AND ($2::text IS NULL OR source_account = $2)
           AND ($3::text IS NULL OR text ILIKE '%' || $3 || '%')
           AND ($4::timestamptz IS NULL OR created_at < $4)
         ORDER BY created_at DESC, event_id DESC
         LIMIT $5",
    )
    .bind(query.platform.as_deref())
    .bind(query.account.as_deref())
    .bind(query.q.as_deref())
    .bind(query.before)
    .bind(limit + 1)
    .fetch_all(&state.db)
    .await;

    match rows {
        Ok(mut items) => {
            let has_more = items.len() > limit as usize;
            items.truncate(limit as usize);
            let next_before = items.last().map(|item| item.created_at);
            Json(serde_json::json!({
                "items": items,
                "next_before": next_before,
                "has_more": has_more
            }))
        }
        Err(error) => {
            warn!(%error, "social posts query failed");
            Json(
                serde_json::json!({ "items": [], "next_before": null, "has_more": false, "error": "social_posts_unavailable" }),
            )
        }
    }
}

pub async fn run_subscriber(nats_url: String, database: sqlx::PgPool) {
    loop {
        match async_nats::connect(&nats_url).await {
            Ok(client) => {
                info!(
                    subject = "social.posts",
                    "social posts subscriber connected and ready"
                );
                match client.subscribe("social.posts").await {
                    Ok(mut subscription) => {
                        while let Some(message) = subscription.next().await {
                            if let Err(error) = persist_post(&database, &message.payload).await {
                                warn!(
                                    error = ?error,
                                    payload_bytes = message.payload.len(),
                                    "failed to persist social post"
                                );
                            }
                        }
                    }
                    Err(error) => warn!(%error, "failed to subscribe to social.posts"),
                }
            }
            Err(error) => warn!(%error, "social posts NATS connection failed"),
        }
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}

async fn persist_post(pool: &sqlx::PgPool, payload: &[u8]) -> anyhow::Result<()> {
    let post: Value = serde_json::from_slice(payload)?;
    let event_id = post
        .get("event_id")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if event_id.is_empty() {
        anyhow::bail!("social post event_id is missing");
    }
    sqlx::query(
        "INSERT INTO news.social_posts (
            event_id, post_id, platform, source_account, author_username,
            author_display_name, text, url, created_at, fetched_at,
            reply_count, retweet_count, like_count, quote_count, language, media_urls
         ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16)
         ON CONFLICT (event_id) DO UPDATE SET
            text = EXCLUDED.text, fetched_at = EXCLUDED.fetched_at,
            reply_count = EXCLUDED.reply_count, retweet_count = EXCLUDED.retweet_count,
            like_count = EXCLUDED.like_count, quote_count = EXCLUDED.quote_count,
            media_urls = EXCLUDED.media_urls",
    )
    .bind(event_id)
    .bind(
        post.get("post_id")
            .and_then(Value::as_str)
            .unwrap_or_default(),
    )
    .bind(
        post.get("platform")
            .and_then(Value::as_str)
            .unwrap_or_default(),
    )
    .bind(
        post.get("source_account")
            .and_then(Value::as_str)
            .unwrap_or_default(),
    )
    .bind(
        post.get("author_username")
            .and_then(Value::as_str)
            .unwrap_or_default(),
    )
    .bind(
        post.get("author_display_name")
            .and_then(Value::as_str)
            .unwrap_or_default(),
    )
    .bind(post.get("text").and_then(Value::as_str).unwrap_or_default())
    .bind(post.get("url").and_then(Value::as_str).unwrap_or_default())
    .bind(
        post.get("created_at")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("created_at missing"))?
            .parse::<DateTime<Utc>>()?,
    )
    .bind(
        post.get("fetched_at")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("fetched_at missing"))?
            .parse::<DateTime<Utc>>()?,
    )
    .bind(
        post.get("reply_count")
            .and_then(Value::as_i64)
            .unwrap_or_default(),
    )
    .bind(
        post.get("retweet_count")
            .and_then(Value::as_i64)
            .unwrap_or_default(),
    )
    .bind(
        post.get("like_count")
            .and_then(Value::as_i64)
            .unwrap_or_default(),
    )
    .bind(
        post.get("quote_count")
            .and_then(Value::as_i64)
            .unwrap_or_default(),
    )
    .bind(
        post.get("language")
            .and_then(Value::as_str)
            .unwrap_or_default(),
    )
    .bind(SqlxJson(
        post.get("media_urls")
            .cloned()
            .unwrap_or_else(|| serde_json::json!([])),
    ))
    .execute(pool)
    .await?;
    Ok(())
}
