use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub plan: String,
    pub is_active: bool,
    pub email_verified: bool,
    pub verify_token: Option<String>,
    pub password_hash: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub name: String,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyEmailRequest {
    pub token: String,
}

impl User {
    pub async fn find_by_email(db: &PgPool, email: &str) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(db)
            .await
    }

    pub async fn find_by_id(db: &PgPool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(db)
            .await
    }

    /// Create user with optional password (for email/password registration).
    pub async fn create(
        db: &PgPool,
        email: &str,
        name: &str,
        verify_token: &str,
        password: Option<&str>,
    ) -> Result<Self, sqlx::Error> {
        let pw_hash = password.map(|pw| hash_password(pw));

        sqlx::query_as::<_, Self>(
            "INSERT INTO users (email, name, verify_token, password_hash) \
             VALUES ($1, $2, $3, $4) RETURNING *",
        )
        .bind(email)
        .bind(name)
        .bind(verify_token)
        .bind(pw_hash)
        .fetch_one(db)
        .await
    }

    /// Create or find user from OAuth provider (no password needed).
    pub async fn find_or_create_oauth(
        db: &PgPool,
        email: &str,
        name: &str,
        avatar_url: Option<&str>,
    ) -> Result<Self, sqlx::Error> {
        // Try to find existing user by email
        if let Some(existing) = Self::find_by_email(db, email).await? {
            // Update avatar if provided
            if let Some(avatar) = avatar_url {
                let _ = sqlx::query(
                    "UPDATE users SET avatar_url = $1, updated_at = NOW() WHERE id = $2",
                )
                .bind(avatar)
                .bind(existing.id)
                .execute(db)
                .await;
            }
            return Ok(existing);
        }

        // Create new user with email already verified (OAuth provider guarantees it)
        sqlx::query_as::<_, Self>(
            "INSERT INTO users (email, name, email_verified, avatar_url) \
             VALUES ($1, $2, TRUE, $3) RETURNING *",
        )
        .bind(email)
        .bind(name)
        .bind(avatar_url)
        .fetch_one(db)
        .await
    }

    /// Verify password against stored hash.
    pub fn verify_password(&self, password: &str) -> bool {
        let Some(ref hash_str) = self.password_hash else {
            return false;
        };
        let Ok(parsed) = PasswordHash::new(hash_str) else {
            return false;
        };
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok()
    }

    pub async fn verify_email(db: &PgPool, token: &str) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "UPDATE users SET email_verified = TRUE, verify_token = NULL, updated_at = NOW() \
             WHERE verify_token = $1 AND email_verified = FALSE RETURNING *",
        )
        .bind(token)
        .fetch_optional(db)
        .await
    }

    pub async fn update_plan(
        db: &PgPool,
        user_id: Uuid,
        plan: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "UPDATE users SET plan = $1, updated_at = NOW() WHERE id = $2 RETURNING *",
        )
        .bind(plan)
        .bind(user_id)
        .fetch_optional(db)
        .await
    }
}

/// Hash a password using Argon2id.
fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .expect("failed to hash password")
        .to_string()
}
