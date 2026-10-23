use atlsd_auth::jwt;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
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

pub fn create_jwt(user: &User, secret: &str, expiry_days: u64) -> Result<String, StatusCode> {
    jwt::create_jwt(
        user.id.to_string(),
        &user.email,
        &user.plan,
        secret,
        expiry_days,
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub fn decode_jwt(token: &str, secret: &str) -> Option<jwt::JwtClaims> {
    jwt::decode_jwt(token, secret)
}

fn create_oauth_state_with_nonce(
    provider: &str,
    nonce: &str,
    secret: &str,
) -> Result<String, StatusCode> {
    jwt::create_oauth_state_with_nonce(provider, nonce, secret)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

fn validate_oauth_state(provider: &str, state_token: &str, secret: &str) -> bool {
    jwt::validate_oauth_state_provider(provider, state_token, secret)
}

fn validate_oauth_state_with_nonce(
    provider: &str,
    nonce: &str,
    state_token: &str,
    secret: &str,
) -> bool {
    jwt::validate_oauth_state_with_nonce(provider, nonce, state_token, secret)
}

fn verified_google_identity(profile: &Value) -> Result<(&str, &str), StatusCode> {
    if profile["verified_email"].as_bool() != Some(true) {
        return Err(StatusCode::FORBIDDEN);
    }
    let id = profile["id"]
        .as_str()
        .filter(|id| !id.is_empty())
        .ok_or(StatusCode::BAD_GATEWAY)?;
    let email = profile["email"]
        .as_str()
        .filter(|email| !email.is_empty())
        .ok_or(StatusCode::BAD_GATEWAY)?;
    Ok((id, email))
}

fn json_with_cookie(body: Value, token: &str, expiry_days: u64) -> Response {
    let mut resp = axum::Json(body).into_response();
    let max_age = expiry_days * 24 * 60 * 60;

    let cookie = format!(
        "wi_jwt={}; HttpOnly; SameSite=Lax; Path=/; Max-Age={}",
        token, max_age
    );

    if let Ok(hv) = cookie.parse() {
        resp.headers_mut()
            .insert(axum::http::header::SET_COOKIE, hv);
    }
    resp
}

pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<CreateUserRequest>,
) -> Result<Response, StatusCode> {
    let email = body.email.trim().to_lowercase();
    if email.is_empty() || !email.contains('@') {
        return Err(StatusCode::BAD_REQUEST);
    }
    let name = body.name.trim().to_string();
    if name.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let password = body.password.as_deref().unwrap_or("");
    if password.len() < 6 {
        return Err(StatusCode::BAD_REQUEST);
    }

    if let Ok(Some(_)) = User::find_by_email(&state.db, &email).await {
        return Err(StatusCode::CONFLICT);
    }

    let verify_token = format!("{}", Uuid::new_v4());

    let user = User::create(&state.db, &email, &name, &verify_token, Some(password))
        .await
        .map_err(|e| {
            warn!(error = %e, "failed to create user");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let default_scopes = vec![
        "market:read".to_string(),
        "realtime:ws".to_string(),
        "news:read".to_string(),
        "macro:read".to_string(),
        "social:read".to_string(),
    ];
    let (_key, raw_key) = ApiKey::create(&state.db, user.id, "default", &default_scopes)
        .await
        .map_err(|e| {
            warn!(error = %e, "failed to create initial API key");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!(user_id = %user.id, email = %email, "new user registered");

    sync::publish_config_changed_for_user(
        &state.redis,
        &state.config.redis_channel_prefix,
        Some(user.id),
    )
    .await;

    let token = create_jwt(
        &user,
        &state.config.jwt_secret,
        state.config.jwt_expiry_days,
    )?;

    let response_body = json!({
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
        "message": "Registration successful. Save your API key — it will only be shown once."
    });

    Ok(json_with_cookie(
        response_body,
        &token,
        state.config.jwt_expiry_days,
    ))
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Response, StatusCode> {
    let email = body.email.trim().to_lowercase();
    if email.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let user = User::find_by_email(&state.db, &email)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let Some(user) = user else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    if !user.verify_password(&body.password) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    if !user.is_active {
        return Err(StatusCode::FORBIDDEN);
    }

    let token = create_jwt(
        &user,
        &state.config.jwt_secret,
        state.config.jwt_expiry_days,
    )?;

    info!(user_id = %user.id, email = %user.email, "user logged in via email/password");

    let response_body = json!({
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

    Ok(json_with_cookie(
        response_body,
        &token,
        state.config.jwt_expiry_days,
    ))
}

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
        None => Ok(Json(
            json!({ "error": "Invalid or expired verification token" }),
        )),
    }
}

pub async fn logout() -> Response {
    let mut resp = axum::Json(json!({ "message": "Logged out successfully" })).into_response();
    let cookie = "wi_jwt=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0";
    if let Ok(hv) = cookie.parse() {
        resp.headers_mut()
            .insert(axum::http::header::SET_COOKIE, hv);
    }
    resp
}

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

    let plan = crate::models::plan::Plan::find_by_id(&state.db, &user.plan)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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

#[derive(serde::Deserialize)]
pub struct UpdateProfileRequest {
    pub name: String,
}

/// PUT /api/v1/auth/profile
pub async fn update_profile(
    State(state): State<AppState>,
    request: axum::extract::Request,
) -> Result<Json<Value>, StatusCode> {
    let auth = request
        .extensions()
        .get::<AuthContext>()
        .cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let bytes = axum::body::to_bytes(request.into_body(), 16 * 1024)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let body: UpdateProfileRequest =
        serde_json::from_slice(&bytes).map_err(|_| StatusCode::BAD_REQUEST)?;

    let trimmed = body.name.trim();
    if trimmed.is_empty() || trimmed.len() > 100 {
        return Err(StatusCode::BAD_REQUEST);
    }

    let user = User::update_name(&state.db, auth.user_id, trimmed)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(json!({
        "status": "ok",
        "user": {
            "id": user.id,
            "email": user.email,
            "name": user.name,
            "plan": user.plan,
            "avatar_url": user.avatar_url,
        }
    })))
}

#[derive(serde::Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: Option<String>,
    pub new_password: String,
}

/// POST /api/v1/auth/change-password
pub async fn change_password(
    State(state): State<AppState>,
    request: axum::extract::Request,
) -> Result<Json<Value>, StatusCode> {
    let auth = request
        .extensions()
        .get::<AuthContext>()
        .cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let bytes = axum::body::to_bytes(request.into_body(), 16 * 1024)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let body: ChangePasswordRequest =
        serde_json::from_slice(&bytes).map_err(|_| StatusCode::BAD_REQUEST)?;

    if body.new_password.len() < 6 {
        return Err(StatusCode::BAD_REQUEST);
    }

    let user = User::find_by_id(&state.db, auth.user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if user.password_hash.is_some() {
        let Some(current) = body.current_password else {
            return Err(StatusCode::BAD_REQUEST);
        };
        if !user.verify_password(&current) {
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    User::update_password(&state.db, auth.user_id, &body.new_password)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!(user_id = %auth.user_id, "user password changed");

    Ok(Json(json!({
        "status": "ok",
        "message": "Password changed successfully"
    })))
}

pub async fn oauth_url(
    State(state): State<AppState>,
    Path(provider): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let redirect_uri = format!("{}/auth/callback/{}", state.config.frontend_url, provider);
    oauth_url_for_redirect(&state, &provider, &redirect_uri, "").await
}

pub async fn desktop_oauth_url(
    State(state): State<AppState>,
    Json(body): Json<DesktopOAuthStart>,
) -> Result<Json<Value>, StatusCode> {
    if body.nonce.len() < 32 || body.nonce.len() > 256 {
        return Err(StatusCode::BAD_REQUEST);
    }
    if !state.config.has_google_oauth() {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
    let redirect_uri = format!(
        "{}/api/v1/auth/oauth/google/desktop-callback",
        state.config.public_api_url
    );
    oauth_url_for_redirect(&state, "google", &redirect_uri, &body.nonce).await
}

async fn oauth_url_for_redirect(
    state: &AppState,
    provider: &str,
    redirect_uri: &str,
    nonce: &str,
) -> Result<Json<Value>, StatusCode> {
    let state_token = create_oauth_state_with_nonce(provider, nonce, &state.config.jwt_secret)?;

    match provider {
        "google" => {
            if !state.config.has_google_oauth() {
                return Ok(Json(json!({ "error": "Google OAuth not configured" })));
            }
            let url = format!(
                "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope=openid%20email%20profile&access_type=offline&state={}",
                state.config.google_client_id,
                urlencoding::encode(redirect_uri),
                urlencoding::encode(&state_token),
            );
            Ok(Json(json!({ "url": url, "state": state_token })))
        }
        "github" => {
            if !state.config.has_github_oauth() {
                return Ok(Json(json!({ "error": "GitHub OAuth not configured" })));
            }
            let url = format!(
                "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope=user:email&state={}",
                state.config.github_client_id,
                urlencoding::encode(redirect_uri),
                urlencoding::encode(&state_token),
            );
            Ok(Json(json!({ "url": url, "state": state_token })))
        }
        _ => Ok(Json(json!({ "error": "Unsupported provider" }))),
    }
}

#[derive(Deserialize)]
pub struct DesktopOAuthStart {
    nonce: String,
}

#[derive(Deserialize)]
pub struct DesktopOAuthCallback {
    code: String,
    state: String,
    nonce: String,
    redirect_uri: String,
}

pub async fn desktop_oauth_callback(
    State(state): State<AppState>,
    Json(body): Json<DesktopOAuthCallback>,
) -> Result<Json<Value>, StatusCode> {
    if !validate_oauth_state_with_nonce(
        "google",
        &body.nonce,
        &body.state,
        &state.config.jwt_secret,
    ) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let redirect = url::Url::parse(&body.redirect_uri).map_err(|_| StatusCode::BAD_REQUEST)?;
    if redirect.scheme() != "http"
        || redirect.host_str() != Some("127.0.0.1")
        || redirect.port() != Some(43827)
        || redirect.path() != "/oauth/callback"
        || redirect.query().is_some()
        || redirect.fragment().is_some()
        || redirect.username() != ""
        || redirect.password().is_some()
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let redirect_uri = body.redirect_uri;
    let client = reqwest::Client::new();
    let token_res = client
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("code", body.code.as_str()),
            ("client_id", state.config.google_client_id.as_str()),
            ("client_secret", state.config.google_client_secret.as_str()),
            ("redirect_uri", redirect_uri.as_str()),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    if !token_res.status().is_success() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let token_data: Value = token_res
        .json()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    let google_access_token = token_data["access_token"]
        .as_str()
        .ok_or(StatusCode::BAD_GATEWAY)?;
    let profile_res = client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(google_access_token)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    if !profile_res.status().is_success() {
        return Err(StatusCode::BAD_GATEWAY);
    }
    let profile: Value = profile_res
        .json()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    let (google_id, email) = verified_google_identity(&profile)?;
    let name = profile["name"].as_str().unwrap_or(email);
    let avatar = profile["picture"].as_str();
    let user = complete_oauth_user(&state, "google", google_id, email, name, avatar).await?;
    let existing_keys = ApiKey::list_by_user(&state.db, user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let active_desktop_keys: Vec<_> = existing_keys
        .iter()
        .filter(|key| key.label == "pia-desktop" && key.is_active)
        .collect();
    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    // ponytail: keep existing keys active so saved desktop configs/env don't break on re-login
    if active_desktop_keys.len() >= 10 {
        for key in &active_desktop_keys[9..] {
            sqlx::query("UPDATE api_keys SET is_active = FALSE WHERE id = $1 AND user_id = $2")
                .bind(key.id)
                .bind(user.id)
                .execute(&mut *tx)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }
    }
    sqlx::query("UPDATE users SET plan = 'enterprise', updated_at = NOW() WHERE id = $1")
        .bind(user.id)
        .execute(&mut *tx)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let desktop_credential = create_desktop_key_in_transaction(&mut tx, user.id).await?;
    tx.commit()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    sync::publish_config_changed_for_user(
        &state.redis,
        &state.config.redis_channel_prefix,
        Some(user.id),
    )
    .await;
    // A generated key is returned only once; stored desktop hashes are not recoverable.
    Ok(Json(json!({
        "desktop_credential": desktop_credential,
        "user": {
            "id": user.id,
            "email": user.email,
            "name": user.name,
            "plan": "enterprise",
            "avatar_url": user.avatar_url
        }
    })))
}

#[derive(Deserialize)]
pub struct OAuthCallbackBody {
    pub code: String,
    pub state: String,
}

pub async fn oauth_callback(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    Json(body): Json<OAuthCallbackBody>,
) -> Result<Response, StatusCode> {
    let redirect_uri = format!("{}/auth/callback/{}", state.config.frontend_url, provider);
    if !validate_oauth_state(&provider, &body.state, &state.config.jwt_secret) {
        return Ok(Json(json!({ "error": "Invalid OAuth state" })).into_response());
    }

    match provider.as_str() {
        "google" => handle_google_oauth(&state, &body.code, &redirect_uri).await,
        "github" => handle_github_oauth(&state, &body.code, &redirect_uri).await,
        _ => Ok(Json(json!({ "error": "Unsupported provider" })).into_response()),
    }
}

async fn handle_google_oauth(
    state: &AppState,
    code: &str,
    redirect_uri: &str,
) -> Result<Response, StatusCode> {
    let client = reqwest::Client::new();

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

    let user_res = client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    let google_user: Value = user_res.json().await.map_err(|_| StatusCode::BAD_GATEWAY)?;

    let (google_id, email) = verified_google_identity(&google_user)?;
    let name = google_user["name"].as_str().unwrap_or(email);
    let avatar = google_user["picture"].as_str();

    complete_oauth_flow(state, "google", google_id, email, name, avatar).await
}

async fn handle_github_oauth(
    state: &AppState,
    code: &str,
    redirect_uri: &str,
) -> Result<Response, StatusCode> {
    let client = reqwest::Client::new();

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

    let token_data: Value = token_res
        .json()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    let access_token = token_data["access_token"]
        .as_str()
        .ok_or(StatusCode::BAD_GATEWAY)?;

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

        let emails: Vec<Value> = emails_res
            .json()
            .await
            .map_err(|_| StatusCode::BAD_GATEWAY)?;
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
        ).into_response());
    }

    complete_oauth_flow(state, "github", &gh_id_str, &email, name, avatar).await
}

async fn complete_oauth_user(
    state: &AppState,
    provider: &str,
    provider_id: &str,
    email: &str,
    name: &str,
    avatar_url: Option<&str>,
) -> Result<User, StatusCode> {
    let linked = OAuthAccount::find_by_provider(&state.db, provider, provider_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user = if let Some(linked) = linked {
        User::find_by_id(&state.db, linked.user_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::UNAUTHORIZED)?
    } else {
        if User::find_by_email(&state.db, email)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .is_some()
        {
            return Err(StatusCode::CONFLICT);
        }
        User::find_or_create_oauth(&state.db, email, name, avatar_url)
            .await
            .map_err(|e| {
                warn!(error = %e, "oauth user creation failed");
                StatusCode::INTERNAL_SERVER_ERROR
            })?
    };
    if !user.is_active {
        return Err(StatusCode::FORBIDDEN);
    }
    let account = OAuthAccount::create(
        &state.db,
        user.id,
        provider,
        provider_id,
        Some(email),
        None,
        &state.config.encryption_key,
    )
    .await
    .map_err(|e| {
        warn!(error = %e, "oauth account linking failed");
        if e.to_string().contains("unique constraint") || e.to_string().contains("duplicate key") {
            StatusCode::CONFLICT
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;
    if account.user_id != user.id {
        return Err(StatusCode::CONFLICT);
    }
    Ok(user)
}

async fn complete_oauth_flow(
    state: &AppState,
    provider: &str,
    provider_id: &str,
    email: &str,
    name: &str,
    avatar_url: Option<&str>,
) -> Result<Response, StatusCode> {
    let user = complete_oauth_user(state, provider, provider_id, email, name, avatar_url).await?;

    let existing_keys = ApiKey::list_by_user(&state.db, user.id)
        .await
        .map_err(|e| {
            warn!(error = %e, "failed to list API keys for OAuth user");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    let active_desktop_keys: Vec<_> = existing_keys
        .iter()
        .filter(|key| key.label == "pia-desktop" && key.is_active)
        .collect();
    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    // ponytail: keep existing keys active so saved desktop configs/env don't break on re-login
    if active_desktop_keys.len() >= 10 {
        for key in &active_desktop_keys[9..] {
            sqlx::query("UPDATE api_keys SET is_active = FALSE WHERE id = $1 AND user_id = $2")
                .bind(key.id)
                .bind(user.id)
                .execute(&mut *tx)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }
    }
    let raw_api_key = create_desktop_key_in_transaction(&mut tx, user.id).await?;
    tx.commit()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    sync::publish_config_changed_for_user(
        &state.redis,
        &state.config.redis_channel_prefix,
        Some(user.id),
    )
    .await;

    let token = create_jwt(
        &user,
        &state.config.jwt_secret,
        state.config.jwt_expiry_days,
    )?;

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

    response["desktop_credential"] = json!(raw_api_key);

    Ok(json_with_cookie(
        response,
        &token,
        state.config.jwt_expiry_days,
    ))
}

async fn create_desktop_key_in_transaction(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: Uuid,
) -> Result<String, StatusCode> {
    use atlsd_auth::api_key::{extract_prefix, generate_raw_key, hash_key};

    let raw = generate_raw_key();
    sqlx::query(
        "INSERT INTO api_keys (user_id, key_hash, key_prefix, label, permissions) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(hash_key(&raw))
    .bind(extract_prefix(&raw))
    .bind("pia-desktop")
    .bind([
        "market:read",
        "realtime:ws",
        "news:read",
        "macro:read",
        "social:read",
    ])
    .execute(&mut **tx)
    .await
    .map_err(|e| {
        warn!(error = %e, "failed to create API key for OAuth user");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(raw)
}

#[cfg(test)]
mod tests {
    use super::verified_google_identity;
    use atlsd_auth::jwt::{create_oauth_state_with_nonce, validate_oauth_state_provider};
    use axum::http::StatusCode;
    use serde_json::json;

    #[test]
    fn google_identity_requires_verified_email_and_provider_id() {
        assert_eq!(
            verified_google_identity(
                &json!({"id":"123", "email":"a@example.com", "verified_email":true})
            ),
            Ok(("123", "a@example.com"))
        );
        assert_eq!(
            verified_google_identity(
                &json!({"id":"123", "email":"a@example.com", "verified_email":false})
            ),
            Err(StatusCode::FORBIDDEN)
        );
        assert_eq!(
            verified_google_identity(&json!({"email":"a@example.com", "verified_email":true})),
            Err(StatusCode::BAD_GATEWAY)
        );
    }

    #[test]
    fn oauth_state_is_signed_and_provider_bound() {
        let secret = "test-secret-with-enough-entropy";
        let state = create_oauth_state_with_nonce("github", "nonce", secret).unwrap();

        assert!(validate_oauth_state_provider("github", &state, secret));
        assert!(!validate_oauth_state_provider("google", &state, secret));
        assert!(!validate_oauth_state_provider(
            "github",
            &state,
            "different-secret"
        ));
    }

    #[test]
    fn update_profile_payload_deserialization() {
        let json = r#"{"name":"Alice Smith"}"#;
        let req: super::UpdateProfileRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "Alice Smith");
    }

    #[test]
    fn change_password_payload_deserialization() {
        let json = r#"{"current_password":"oldPass123","new_password":"newPass456"}"#;
        let req: super::ChangePasswordRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.current_password.as_deref(), Some("oldPass123"));
        assert_eq!(req.new_password, "newPass456");
    }
}
