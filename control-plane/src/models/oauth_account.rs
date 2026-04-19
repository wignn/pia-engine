use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OAuthAccount {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
    pub provider_id: String,
    pub provider_email: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl OAuthAccount {
    /// Find an OAuth account by provider + provider_id.
    pub async fn find_by_provider(
        db: &PgPool,
        provider: &str,
        provider_id: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM oauth_accounts WHERE provider = $1 AND provider_id = $2",
        )
        .bind(provider)
        .bind(provider_id)
        .fetch_optional(db)
        .await
    }

    /// Create a new OAuth account link.
    pub async fn create(
        db: &PgPool,
        user_id: Uuid,
        provider: &str,
        provider_id: &str,
        provider_email: Option<&str>,
        access_token: Option<&str>,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "INSERT INTO oauth_accounts (user_id, provider, provider_id, provider_email, access_token) \
             VALUES ($1, $2, $3, $4, $5) \
             ON CONFLICT (provider, provider_id) DO UPDATE SET access_token = $5 \
             RETURNING *",
        )
        .bind(user_id)
        .bind(provider)
        .bind(provider_id)
        .bind(provider_email)
        .bind(access_token)
        .fetch_one(db)
        .await
    }

    /// List all OAuth accounts for a user.
    pub async fn list_by_user(db: &PgPool, user_id: Uuid) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM oauth_accounts WHERE user_id = $1 ORDER BY created_at",
        )
        .bind(user_id)
        .fetch_all(db)
        .await
    }
}
