use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{info, warn};
use uuid::Uuid;

use crate::api::server::AuthContext;
use crate::api::AppState;
use crate::models::{
    api_key::ApiKey,
    oauth_account::OAuthAccount,
    user::{CreateUserRequest, LoginRequest, User, VerifyEmailRequest},
};
use crate::sync;

// ─── JWT Claims ──────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JwtClaims {
    pub sub: String,   // user_id
    pub email: String,
    pub plan: String,
    pub exp: usize,
    pub iat: usize,
}

/// Create a signed JWT for a user.
pub fn create_jwt(user: &User, secret: &str, expiry_days: u64) -> Result<String, StatusCode> {
    let now = Utc::now();
    let exp = now + Duration::days(expiry_days as i64);

    let claims = JwtClaims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        plan: user.plan.clone(),
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| {
        warn!(error = %e, "failed to create JWT");
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

/// Decode and validate a JWT. Returns claims if valid.
pub fn decode_jwt(token: &str, secret: &str) -> Option<JwtClaims> {
    decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .ok()
    .map(|data| data.claims)
}

// ─── POST /api/v1/auth/register ──────────────────────────────────────────────

pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<CreateUserRequest>,
) -> Result<Json<Value>, StatusCode> {
    let email = body.email.trim().to_lowercase();
    if email.is_empty() || !email.contains('@') {
        return Ok(Json(json!({ "error": "Invalid email address" })));
    }
    let name = body.name.trim().to_string();
    if name.is_empty() {
        return Ok(Json(json!({ "error": "Name is required" })));
    }

    // Validate password
    let password = body.password.as_deref().unwrap_or("");
    if password.len() < 6 {
        return Ok(Json(json!({ "error": "Password must be at least 6 characters" })));
    }

    // Check existing
    if let Ok(Some(_)) = User::find_by_email(&state.db, &email).await {
        return Ok(Json(json!({ "error": "Email already registered" })));
    }

    // Generate verify token
    let verify_token = format!("{}", Uuid::new_v4());

    // Create user with password
    let user = User::create(&state.db, &email, &name, &verify_token, Some(password))
        .await
        .map_err(|e| {
            warn!(error = %e, "failed to create user");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Generate first API key
    let (_key, raw_key) = ApiKey::create(&state.db, user.id, "default", &[])
        .await
        .map_err(|e| {
            warn!(error = %e, "failed to create initial API key");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!(user_id = %user.id, email = %email, "new user registered");

    // Sync to Redis
    sync::publish_config_changed(&state.redis, &state.config.redis_channel_prefix).await;

    // Issue JWT
    let token = create_jwt(&user, &state.config.jwt_secret, state.config.jwt_expiry_days)?;

    Ok(Json(json!({
        "user": {
            "id": user.id,
            "email": user.email,
            "name": user.name,
            "plan": user.plan,
            "email_verified": user.email_verified,
            "created_at": user.created_at,
        },
        "token": token,
        "api_key": raw_key,
        "verify_token": verify_token,
        "message": "Registration successful. Save your API key — it will only be shown once."
    })))
}

// ─── POST /api/v1/auth/login ─────────────────────────────────────────────────

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<Value>, StatusCode> {
    let email = body.email.trim().to_lowercase();
    if email.is_empty() {
        return Ok(Json(json!({ "error": "Email is required" })));
    }

    let user = User::find_by_email(&state.db, &email)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let Some(user) = user else {
        return Ok(Json(json!({ "error": "Invalid email or password" })));
    };

    if !user.verify_password(&body.password) {
        return Ok(Json(json!({ "error": "Invalid email or password" })));
    }

    if !user.is_active {
        return Ok(Json(json!({ "error": "Account is deactivated" })));
    }

    let token = create_jwt(&user, &state.config.jwt_secret, state.config.jwt_expiry_days)?;

    info!(user_id = %user.id, email = %user.email, "user logged in via email/password");

    Ok(Json(json!({
        "user": {
            "id": user.id,
            "email": user.email,
            "name": user.name,
            "plan": user.plan,
            "email_verified": user.email_verified,
            "avatar_url": user.avatar_url,
            "created_at": user.created_at,
        },
        "token": token,
    })))
}

// ─── POST /api/v1/auth/verify ────────────────────────────────────────────────

pub async fn verify_email(
    State(state): State<AppState>,
    Json(body): Json<VerifyEmailRequest>,
) -> Result<Json<Value>, StatusCode> {
    let token = body.token.trim();
    if token.is_empty() {
        return Ok(Json(json!({ "error": "Token is required" })));
    }

    let user = User::verify_email(&state.db, token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match user {
        Some(u) => {
            info!(user_id = %u.id, email = %u.email, "email verified");
            Ok(Json(json!({
                "message": "Email verified successfully",
                "user": {
                    "id": u.id,
                    "email": u.email,
                    "email_verified": true,
                }
            })))
        }
        None => Ok(Json(json!({ "error": "Invalid or expired verification token" }))),
    }
}

// ─── GET /api/v1/auth/me ─────────────────────────────────────────────────────

pub async fn me(
    State(state): State<AppState>,
    request: axum::extract::Request,
) -> Result<Json<Value>, StatusCode> {
    let auth = request
        .extensions()
        .get::<AuthContext>()
        .cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if auth.is_admin {
        return Ok(Json(json!({
            "id": auth.user_id,
            "role": "admin",
            "plan": "enterprise",
        })));
    }

    let user = User::find_by_id(&state.db, auth.user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let key_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM api_keys WHERE user_id = $1 AND is_active = TRUE")
            .bind(user.id)
            .fetch_one(&state.db)
            .await
            .unwrap_or((0,));

    // Get plan limits
    let plan = crate::models::plan::Plan::find_by_id(&state.db, &user.plan)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get linked OAuth providers
    let oauth_accounts = OAuthAccount::list_by_user(&state.db, user.id)
        .await
        .unwrap_or_default();
    let linked_providers: Vec<String> = oauth_accounts.iter().map(|a| a.provider.clone()).collect();

    Ok(Json(json!({
        "user": {
            "id": user.id,
            "email": user.email,
            "name": user.name,
            "plan": user.plan,
            "email_verified": user.email_verified,
            "avatar_url": user.avatar_url,
            "created_at": user.created_at,
            "has_password": user.password_hash.is_some(),
        },
        "active_keys": key_count.0,
        "plan_limits": plan,
        "linked_providers": linked_providers,
    })))
}

// ─── OAuth: GET /api/v1/auth/oauth/:provider/url ─────────────────────────────

pub async fn oauth_url(
    State(state): State<AppState>,
    Path(provider): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let redirect_uri = format!("{}/auth/callback/{}", state.config.frontend_url, provider);

    match provider.as_str() {
        "google" => {
            if !state.config.has_google_oauth() {
                return Ok(Json(json!({ "error": "Google OAuth not configured" })));
            }
            let url = format!(
                "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope=openid%20email%20profile&access_type=offline",
                state.config.google_client_id,
                urlencoding::encode(&redirect_uri),
            );
            Ok(Json(json!({ "url": url })))
        }
        "github" => {
            if !state.config.has_github_oauth() {
                return Ok(Json(json!({ "error": "GitHub OAuth not configured" })));
            }
            let url = format!(
                "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope=user:email",
                state.config.github_client_id,
                urlencoding::encode(&redirect_uri),
            );
            Ok(Json(json!({ "url": url })))
        }
        _ => Ok(Json(json!({ "error": "Unsupported provider" }))),
    }
}

// ─── OAuth: POST /api/v1/auth/oauth/:provider/callback ──────────────────────

#[derive(Deserialize)]
pub struct OAuthCallbackBody {
    pub code: String,
}

pub async fn oauth_callback(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    Json(body): Json<OAuthCallbackBody>,
) -> Result<Json<Value>, StatusCode> {
    let redirect_uri = format!("{}/auth/callback/{}", state.config.frontend_url, provider);

    match provider.as_str() {
        "google" => handle_google_oauth(&state, &body.code, &redirect_uri).await,
        "github" => handle_github_oauth(&state, &body.code, &redirect_uri).await,
        _ => Ok(Json(json!({ "error": "Unsupported provider" }))),
    }
}

/// Exchange Google auth code for tokens, fetch user info, create/link account.
async fn handle_google_oauth(
    state: &AppState,
    code: &str,
    redirect_uri: &str,
) -> Result<Json<Value>, StatusCode> {
    let client = reqwest::Client::new();

    // Exchange code for tokens
    let token_res = client
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("code", code),
            ("client_id", &state.config.google_client_id),
            ("client_secret", &state.config.google_client_secret),
            ("redirect_uri", redirect_uri),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
        .map_err(|e| {
            warn!(error = %e, "google token exchange failed");
            StatusCode::BAD_GATEWAY
        })?;

    let token_data: Value = token_res.json().await.map_err(|e| {
        warn!(error = %e, "google token parse failed");
        StatusCode::BAD_GATEWAY
    })?;

    let access_token = token_data["access_token"]
        .as_str()
        .ok_or(StatusCode::BAD_GATEWAY)?;

    // Fetch user info
    let user_res = client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    let google_user: Value = user_res.json().await.map_err(|_| StatusCode::BAD_GATEWAY)?;

    let google_id = google_user["id"]
        .as_str()
        .ok_or(StatusCode::BAD_GATEWAY)?;
    let email = google_user["email"]
        .as_str()
        .ok_or(StatusCode::BAD_GATEWAY)?;
    let name = google_user["name"].as_str().unwrap_or(email);
    let avatar = google_user["picture"].as_str();

    complete_oauth_flow(state, "google", google_id, email, name, avatar, Some(access_token)).await
}

/// Exchange GitHub auth code for tokens, fetch user info, create/link account.
async fn handle_github_oauth(
    state: &AppState,
    code: &str,
    redirect_uri: &str,
) -> Result<Json<Value>, StatusCode> {
    let client = reqwest::Client::new();

    // Exchange code for access token
    let token_res = client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&[
            ("client_id", state.config.github_client_id.as_str()),
            ("client_secret", state.config.github_client_secret.as_str()),
            ("code", code),
            ("redirect_uri", redirect_uri),
        ])
        .send()
        .await
        .map_err(|e| {
            warn!(error = %e, "github token exchange failed");
            StatusCode::BAD_GATEWAY
        })?;

    let token_data: Value = token_res.json().await.map_err(|_| StatusCode::BAD_GATEWAY)?;

    let access_token = token_data["access_token"]
        .as_str()
        .ok_or(StatusCode::BAD_GATEWAY)?;

    // Fetch user info
    let user_res = client
        .get("https://api.github.com/user")
        .header("User-Agent", "world-info-portal")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    let gh_user: Value = user_res.json().await.map_err(|_| StatusCode::BAD_GATEWAY)?;

    let gh_id = gh_user["id"].as_i64().ok_or(StatusCode::BAD_GATEWAY)?;
    let gh_id_str = gh_id.to_string();
    let name = gh_user["name"]
        .as_str()
        .or_else(|| gh_user["login"].as_str())
        .unwrap_or("User");
    let avatar = gh_user["avatar_url"].as_str();

    // GitHub may not return email in user profile; fetch from /user/emails
    let email = if let Some(e) = gh_user["email"].as_str() {
        e.to_string()
    } else {
        let emails_res = client
            .get("https://api.github.com/user/emails")
            .header("User-Agent", "world-info-portal")
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|_| StatusCode::BAD_GATEWAY)?;

        let emails: Vec<Value> = emails_res.json().await.map_err(|_| StatusCode::BAD_GATEWAY)?;
        emails
            .iter()
            .find(|e| e["primary"].as_bool() == Some(true))
            .and_then(|e| e["email"].as_str())
            .unwrap_or_default()
            .to_string()
    };

    if email.is_empty() {
        return Ok(Json(
            json!({ "error": "Could not retrieve email from GitHub. Please make your email public or use email/password login." }),
        ));
    }

    complete_oauth_flow(
        state,
        "github",
        &gh_id_str,
        &email,
        name,
        avatar,
        Some(access_token),
    )
    .await
}

/// Shared logic: find/create user, link OAuth account, issue JWT.
async fn complete_oauth_flow(
    state: &AppState,
    provider: &str,
    provider_id: &str,
    email: &str,
    name: &str,
    avatar_url: Option<&str>,
    access_token: Option<&str>,
) -> Result<Json<Value>, StatusCode> {
    // Find or create the user
    let user = User::find_or_create_oauth(&state.db, email, name, avatar_url)
        .await
        .map_err(|e| {
            warn!(error = %e, "oauth user creation failed");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Link OAuth account
    OAuthAccount::create(
        &state.db,
        user.id,
        provider,
        provider_id,
        Some(email),
        access_token,
    )
    .await
    .map_err(|e| {
        warn!(error = %e, "oauth account linking failed");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Ensure user has at least one API key
    let existing_keys = ApiKey::list_by_user(&state.db, user.id)
        .await
        .unwrap_or_default();
    let mut raw_api_key: Option<String> = None;
    if existing_keys.is_empty() {
        if let Ok((_key, raw)) = ApiKey::create(&state.db, user.id, "default", &[]).await {
            raw_api_key = Some(raw);
        }
    }

    // Sync to Redis
    sync::publish_config_changed(&state.redis, &state.config.redis_channel_prefix).await;

    // Issue JWT
    let token = create_jwt(&user, &state.config.jwt_secret, state.config.jwt_expiry_days)?;

    info!(
        user_id = %user.id,
        email = %user.email,
        provider = %provider,
        "user logged in via OAuth"
    );

    let mut response = json!({
        "user": {
            "id": user.id,
            "email": user.email,
            "name": user.name,
            "plan": user.plan,
            "email_verified": user.email_verified,
            "avatar_url": user.avatar_url,
            "created_at": user.created_at,
        },
        "token": token,
    });

    if let Some(key) = raw_api_key {
        response["api_key"] = json!(key);
        response["message"] =
            json!("Welcome! Your first API key has been generated. Save it — it won't be shown again.");
    }

    Ok(Json(response))
}
