use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TenantConfig {
    pub id: Uuid,
    pub user_id: Uuid,
    pub config_key: String,
    pub config_value: serde_json::Value,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SetConfigRequest {
    pub value: serde_json::Value,
}

impl TenantConfig {
    /// Get all configs for a user.
    pub async fn list_by_user(db: &PgPool, user_id: Uuid) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM tenant_configs WHERE user_id = $1 ORDER BY config_key",
        )
        .bind(user_id)
        .fetch_all(db)
        .await
    }

    /// Get a specific config.
    pub async fn get(
        db: &PgPool,
        user_id: Uuid,
        key: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM tenant_configs WHERE user_id = $1 AND config_key = $2",
        )
        .bind(user_id)
        .bind(key)
        .fetch_optional(db)
        .await
    }

    /// Upsert a config value.
    pub async fn set(
        db: &PgPool,
        user_id: Uuid,
        key: &str,
        value: &serde_json::Value,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "INSERT INTO tenant_configs (user_id, config_key, config_value) \
             VALUES ($1, $2, $3) \
             ON CONFLICT (user_id, config_key) \
             DO UPDATE SET config_value = $3, updated_at = NOW() \
             RETURNING *",
        )
        .bind(user_id)
        .bind(key)
        .bind(value)
        .fetch_one(db)
        .await
    }

    /// Delete a config key.
    pub async fn delete(db: &PgPool, user_id: Uuid, key: &str) -> Result<bool, sqlx::Error> {
        let r = sqlx::query(
            "DELETE FROM tenant_configs WHERE user_id = $1 AND config_key = $2",
        )
        .bind(user_id)
        .bind(key)
        .execute(db)
        .await?;
        Ok(r.rows_affected() > 0)
    }

    /// Get all unique X usernames across all tenants (for the core pipeline merger).
    pub async fn all_x_usernames(db: &PgPool) -> Result<Vec<String>, sqlx::Error> {
        let rows: Vec<(serde_json::Value,)> = sqlx::query_as(
            "SELECT config_value FROM tenant_configs WHERE config_key = 'x_usernames'",
        )
        .fetch_all(db)
        .await?;

        let mut all = Vec::new();
        for (val,) in rows {
            if let Some(arr) = val.as_array() {
                for item in arr {
                    if let Some(s) = item.as_str() {
                        let s = s.trim().to_lowercase();
                        if !s.is_empty() && !all.contains(&s) {
                            all.push(s);
                        }
                    }
                }
            }
        }
        Ok(all)
    }
}
