use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

const KEY_PREFIX: &str = "wi_live_";

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub key_hash: String,
    pub key_prefix: String,
    pub label: String,
    pub permissions: Vec<String>,
    pub is_active: bool,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Result of looking up a key — includes the user's plan for convenience.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ApiKeyWithPlan {
    pub key_id: Uuid,
    pub user_id: Uuid,
    pub permissions: Vec<String>,
    pub plan: String,
    pub user_is_active: bool,
    pub key_is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateKeyRequest {
    pub label: Option<String>,
    pub permissions: Option<Vec<String>>,
}

/// Serialized API key info returned to users (never includes the full key or hash).
#[derive(Debug, Serialize)]
pub struct ApiKeyInfo {
    pub id: Uuid,
    pub key_prefix: String,
    pub label: String,
    pub permissions: Vec<String>,
    pub is_active: bool,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<ApiKey> for ApiKeyInfo {
    fn from(k: ApiKey) -> Self {
        Self {
            id: k.id,
            key_prefix: k.key_prefix,
            label: k.label,
            permissions: k.permissions,
            is_active: k.is_active,
            last_used_at: k.last_used_at,
            expires_at: k.expires_at,
            created_at: k.created_at,
        }
    }
}

/// Generate a new raw API key string (e.g., "wi_live_a1b2c3d4...").
pub fn generate_raw_key() -> String {
    let mut rng = rand::thread_rng();
    let bytes: [u8; 24] = rng.gen();
    format!("{}{}", KEY_PREFIX, hex::encode(bytes))
}

/// SHA-256 hash a raw API key for storage.
pub fn hash_key(raw: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    hex::encode(hasher.finalize())
}

/// Extract the prefix portion for display (first 16 chars).
pub fn extract_prefix(raw: &str) -> String {
    if raw.len() >= 16 {
        format!("{}...", &raw[..16])
    } else {
        raw.to_string()
    }
}

impl ApiKey {
    /// Create a new API key for a user. Returns (ApiKey, raw_key_string).
    pub async fn create(
        db: &PgPool,
        user_id: Uuid,
        label: &str,
        permissions: &[String],
    ) -> Result<(Self, String), sqlx::Error> {
        let raw = generate_raw_key();
        let hashed = hash_key(&raw);
        let prefix = extract_prefix(&raw);

        let key = sqlx::query_as::<_, Self>(
            "INSERT INTO api_keys (user_id, key_hash, key_prefix, label, permissions) \
             VALUES ($1, $2, $3, $4, $5) RETURNING *",
        )
        .bind(user_id)
        .bind(&hashed)
        .bind(&prefix)
        .bind(label)
        .bind(permissions)
        .fetch_one(db)
        .await?;

        Ok((key, raw))
    }

    /// List all API keys for a user.
    pub async fn list_by_user(db: &PgPool, user_id: Uuid) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM api_keys WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(db)
        .await
    }

    /// Lookup key by its hash — used for authentication.
    pub async fn find_by_hash(db: &PgPool, key_hash: &str) -> Result<Option<ApiKeyWithPlan>, sqlx::Error> {
        sqlx::query_as::<_, ApiKeyWithPlan>(
            "SELECT k.id AS key_id, k.user_id, k.permissions, u.plan, \
                    u.is_active AS user_is_active, k.is_active AS key_is_active \
             FROM api_keys k \
             JOIN users u ON u.id = k.user_id \
             WHERE k.key_hash = $1",
        )
        .bind(key_hash)
        .fetch_optional(db)
        .await
    }

    /// Revoke (deactivate) a key.
    pub async fn revoke(db: &PgPool, key_id: Uuid, user_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE api_keys SET is_active = FALSE WHERE id = $1 AND user_id = $2",
        )
        .bind(key_id)
        .bind(user_id)
        .execute(db)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Update label.
    pub async fn update_label(
        db: &PgPool,
        key_id: Uuid,
        user_id: Uuid,
        label: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE api_keys SET label = $1 WHERE id = $2 AND user_id = $3",
        )
        .bind(label)
        .bind(key_id)
        .bind(user_id)
        .execute(db)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Touch last_used_at timestamp.
    pub async fn touch(db: &PgPool, key_id: Uuid) {
        let _ = sqlx::query("UPDATE api_keys SET last_used_at = NOW() WHERE id = $1")
            .bind(key_id)
            .execute(db)
            .await;
    }
}
