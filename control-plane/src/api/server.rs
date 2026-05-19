use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
    routing::{delete, get, patch, post, put},
    Router,
};
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing::info;

use super::AppState;
use crate::api::auth;
use crate::models::{api_key, user::User};

/// Authenticates requests and stores the resolved principal in request extensions.
///
/// Accepted credentials are JWT bearer tokens, `X-API-Key`, and `token` or
/// `api_key` query parameters for clients that cannot set custom headers.
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = request.uri().path();

    if path == "/health"
        || path == "/"
        || path == "/api/v1/auth/register"
        || path == "/api/v1/auth/login"
        || path == "/api/v1/auth/verify"
        || path == "/api/v1/plans"
        || path.starts_with("/api/v1/auth/oauth/")
        || path.starts_with("/portal")
        || path.starts_with("/assets")
    {
        return Ok(next.run(request).await);
    }

    let raw_key = extract_key(&request);
    let bearer_token = extract_bearer(&request);

    if let Some(token) = bearer_token {
        if let Some(claims) = auth::decode_jwt(&token, &state.config.jwt_secret) {
            if let Ok(user_id) = claims.sub.parse::<uuid::Uuid>() {
                let user = User::find_by_id(&state.db, user_id)
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                let Some(user) = user else {
                    return Err(StatusCode::UNAUTHORIZED);
                };
                if !user.is_active {
                    return Err(StatusCode::FORBIDDEN);
                }
                request.extensions_mut().insert(AuthContext {
                    user_id,
                    key_id: uuid::Uuid::nil(),
                    plan: user.plan,
                    is_admin: false,
                });
                return Ok(next.run(request).await);
            }
        }
    }

    if let Some(raw) = raw_key {
        if !state.config.admin_api_key.is_empty() && raw == state.config.admin_api_key {
            request.extensions_mut().insert(AuthContext {
                user_id: uuid::Uuid::nil(),
                key_id: uuid::Uuid::nil(),
                plan: "enterprise".to_string(),
                is_admin: true,
            });
            return Ok(next.run(request).await);
        }

        let hashed = api_key::hash_key(&raw);
        let lookup = api_key::ApiKey::find_by_hash(&state.db, &hashed)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if let Some(key_info) = lookup {
            if !key_info.key_is_active || !key_info.user_is_active {
                return Err(StatusCode::FORBIDDEN);
            }

            let db = state.db.clone();
            let kid = key_info.key_id;
            tokio::spawn(async move { api_key::ApiKey::touch(&db, kid).await });

            request.extensions_mut().insert(AuthContext {
                user_id: key_info.user_id,
                key_id: key_info.key_id,
                plan: key_info.plan,
                is_admin: false,
            });
            return Ok(next.run(request).await);
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}

/// Extracts an API key from headers or supported query parameters.
fn extract_key(request: &Request) -> Option<String> {
    if let Some(val) = request.headers().get("X-API-Key") {
        if let Ok(s) = val.to_str() {
            if !s.is_empty() {
                return Some(s.to_string());
            }
        }
    }

    let uri = request.uri();
    uri.query().and_then(|q| {
        url::form_urlencoded::parse(q.as_bytes())
            .find(|(k, _)| k == "token" || k == "api_key")
            .map(|(_, v)| v.to_string())
    })
}

fn extract_bearer(request: &Request) -> Option<String> {
    if let Some(val) = request.headers().get(header::AUTHORIZATION) {
        if let Ok(s) = val.to_str() {
            if let Some(token) = s.strip_prefix("Bearer ") {
                if !token.is_empty() {
                    return Some(token.to_string());
                }
            }
        }
    }

    if let Some(val) = request.headers().get(header::COOKIE) {
        if let Ok(s) = val.to_str() {
            for cookie in s.split(';') {
                let cookie = cookie.trim();
                if let Some(token) = cookie.strip_prefix("wi_jwt=") {
                    if !token.is_empty() {
                        return Some(token.to_string());
                    }
                }
            }
        }
    }

    None
}

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: uuid::Uuid,
    pub key_id: uuid::Uuid,
    pub plan: String,
    pub is_admin: bool,
}

pub fn build_router(state: AppState) -> Router {
    let allowed_origins: Vec<axum::http::HeaderValue> = std::env::var("ALLOWED_ORIGINS")
        .unwrap_or_else(|_| {
            "http://localhost:3000,http://localhost:5173,http://localhost:8080".to_string()
        })
        .split(',')
        .filter_map(|o| o.trim().parse().ok())
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(allowed_origins))
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::PATCH,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            axum::http::HeaderName::from_static("x-api-key"),
        ])
        .allow_credentials(true);

    Router::new()
        .route("/health", get(health))
        .route("/", get(root))
        .route("/api/v1/auth/register", post(super::auth::register))
        .route("/api/v1/auth/login", post(super::auth::login))
        .route("/api/v1/auth/verify", post(super::auth::verify_email))
        .route("/api/v1/plans", get(super::plans::list_plans))
        .route(
            "/api/v1/auth/oauth/{provider}/url",
            get(super::auth::oauth_url),
        )
        .route(
            "/api/v1/auth/oauth/{provider}/callback",
            post(super::auth::oauth_callback),
        )
        .route("/api/v1/auth/me", get(super::auth::me))
        .route("/api/v1/keys", get(super::keys::list_keys))
        .route("/api/v1/keys", post(super::keys::create_key))
        .route("/api/v1/keys/{id}", delete(super::keys::revoke_key))
        .route("/api/v1/keys/{id}", patch(super::keys::update_key))
        .route("/api/v1/config", get(super::tenant_config::list_config))
        .route(
            "/api/v1/config/{key}",
            put(super::tenant_config::set_config),
        )
        .route(
            "/api/v1/config/{key}",
            delete(super::tenant_config::delete_config),
        )
        .route("/api/v1/usage", get(super::usage::summary))
        .route("/api/v1/usage/history", get(super::usage::history))
        .route("/api/v1/plans/upgrade", post(super::plans::upgrade))
        .route("/api/v1/admin/users", get(super::admin::list_users))
        .route(
            "/api/v1/admin/users/{id}/plan",
            post(super::admin::set_user_plan),
        )
        .route(
            "/api/v1/admin/users/{id}/toggle",
            post(super::admin::toggle_user),
        )
        .route("/api/v1/admin/stats", get(super::admin::platform_stats))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .layer(cors)
        .with_state(state)
}

async fn health() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "healthy",
        "service": "world-info-control-plane",
    }))
}

async fn root() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "service": "World Info Control Plane",
        "version": "1.0.0",
    }))
}

pub async fn start(state: AppState) -> Result<(), Box<dyn std::error::Error>> {
    let port = state.config.port;
    let app = build_router(state);
    let addr = format!("0.0.0.0:{}", port);

    info!(addr = %addr, "control-plane HTTP server starting");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("received Ctrl+C, shutting down"),
        _ = terminate => info!("received SIGTERM, shutting down"),
    }
}
