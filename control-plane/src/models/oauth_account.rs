use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::crypto;

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

    /// Create a new OAuth account link. Access token is encrypted before storage.
    pub async fn create(
        db: &PgPool,
        user_id: Uuid,
        provider: &str,
        provider_id: &str,
        provider_email: Option<&str>,
        access_token: Option<&str>,
        encryption_key: &str,
    ) -> Result<Self, sqlx::Error> {
        // Encrypt the access token if provided
        let encrypted_token = access_token.and_then(|t| {
            crypto::encrypt(t, encryption_key)
                .map_err(|e| tracing::warn!(error = %e, "failed to encrypt OAuth token"))
                .ok()
        });

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
        .bind(encrypted_token.as_deref())
        .fetch_one(db)
        .await
    }

    /// Decrypt the stored access token.
    pub fn decrypt_access_token(&self, encryption_key: &str) -> Option<String> {
        self.access_token.as_ref().and_then(|t| {
            crypto::decrypt(t, encryption_key)
                .map_err(|e| tracing::warn!(error = %e, "failed to decrypt OAuth token"))
                .ok()
        })
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
