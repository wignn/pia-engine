use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Plan {
    pub id: String,
    pub name: String,
    pub price_idr: i64,
    pub requests_per_day: i32,
    pub ws_connections: i32,
    pub x_usernames_max: i32,
    pub tv_symbols_max: i32,
    pub news_history_days: i32,
    pub rate_limit_per_min: i32,
    pub can_scrape: bool,
    pub can_custom_rss: bool,
    pub is_active: bool,
    pub sort_order: i32,
}

impl Plan {
    pub async fn list_active(db: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM plans WHERE is_active = TRUE ORDER BY sort_order ASC",
        )
        .fetch_all(db)
        .await
    }

    pub async fn find_by_id(db: &PgPool, plan_id: &str) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM plans WHERE id = $1")
            .bind(plan_id)
            .fetch_optional(db)
            .await
    }
}
