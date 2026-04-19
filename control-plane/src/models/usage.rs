use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UsageLog {
    pub id: Uuid,
    pub user_id: Uuid,
    pub api_key_id: Option<Uuid>,
    pub endpoint: String,
    pub method: String,
    pub status_code: i32,
    pub response_ms: Option<i32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct UsageSummary {
    pub today: i64,
    pub this_week: i64,
    pub this_month: i64,
    pub plan_limit: i32,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct DailyUsage {
    pub day: String,
    pub count: i64,
}

impl UsageLog {
    /// Record a usage event (fire-and-forget).
    pub async fn record(
        db: &PgPool,
        user_id: Uuid,
        api_key_id: Option<Uuid>,
        endpoint: &str,
        method: &str,
        status_code: i32,
        response_ms: Option<i32>,
    ) {
        let _ = sqlx::query(
            "INSERT INTO usage_logs (user_id, api_key_id, endpoint, method, status_code, response_ms) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(user_id)
        .bind(api_key_id)
        .bind(endpoint)
        .bind(method)
        .bind(status_code)
        .bind(response_ms)
        .execute(db)
        .await;
    }

    /// Count requests for today.
    pub async fn count_today(db: &PgPool, user_id: Uuid) -> Result<i64, sqlx::Error> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM usage_logs WHERE user_id = $1 AND created_at >= CURRENT_DATE",
        )
        .bind(user_id)
        .fetch_one(db)
        .await?;
        Ok(count.0)
    }

    /// Get usage summary.
    pub async fn summary(db: &PgPool, user_id: Uuid) -> Result<(i64, i64, i64), sqlx::Error> {
        let today: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM usage_logs WHERE user_id = $1 AND created_at >= CURRENT_DATE",
        )
        .bind(user_id)
        .fetch_one(db)
        .await?;

        let week: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM usage_logs WHERE user_id = $1 AND created_at >= date_trunc('week', CURRENT_DATE)",
        )
        .bind(user_id)
        .fetch_one(db)
        .await?;

        let month: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM usage_logs WHERE user_id = $1 AND created_at >= date_trunc('month', CURRENT_DATE)",
        )
        .bind(user_id)
        .fetch_one(db)
        .await?;

        Ok((today.0, week.0, month.0))
    }

    /// Daily breakdown for the last N days.
    pub async fn daily_history(db: &PgPool, user_id: Uuid, days: i32) -> Result<Vec<DailyUsage>, sqlx::Error> {
        sqlx::query_as::<_, DailyUsage>(
            "SELECT to_char(created_at::date, 'YYYY-MM-DD') AS day, COUNT(*) AS count \
             FROM usage_logs \
             WHERE user_id = $1 AND created_at >= CURRENT_DATE - ($2 || ' days')::interval \
             GROUP BY created_at::date \
             ORDER BY created_at::date DESC",
        )
        .bind(user_id)
        .bind(days)
        .fetch_all(db)
        .await
    }
}
