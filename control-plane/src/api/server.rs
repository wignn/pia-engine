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
use crate::models::api_key;

/// Middleware: authenticate requests using JWT Bearer token, X-API-Key header, or ?token= query param.
/// Injects AuthContext into request extensions.
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = request.uri().path();

    // Public endpoints (no auth required)
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

    // Try to extract auth from the request
    let raw_key = extract_key(&request);
    let bearer_token = extract_bearer(&request);

    // 1. Try JWT Bearer token first
    if let Some(token) = bearer_token {
        if let Some(claims) = auth::decode_jwt(&token, &state.config.jwt_secret) {
            if let Ok(user_id) = claims.sub.parse::<uuid::Uuid>() {
                request.extensions_mut().insert(AuthContext {
                    user_id,
                    key_id: uuid::Uuid::nil(),
                    plan: claims.plan,
                    is_admin: false,
                });
                return Ok(next.run(request).await);
            }
        }
    }

    // 2. Try API key
    if let Some(raw) = raw_key {
        // Admin bypass
        if !state.config.admin_api_key.is_empty() && raw == state.config.admin_api_key {
            request.extensions_mut().insert(AuthContext {
                user_id: uuid::Uuid::nil(),
                key_id: uuid::Uuid::nil(),
                plan: "enterprise".to_string(),
                is_admin: true,
            });
            return Ok(next.run(request).await);
        }

        // Lookup in DB
        let hashed = api_key::hash_key(&raw);
        let lookup = api_key::ApiKey::find_by_hash(&state.db, &hashed)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if let Some(key_info) = lookup {
            if !key_info.key_is_active || !key_info.user_is_active {
                return Err(StatusCode::FORBIDDEN);
            }

            // Touch last_used_at (fire-and-forget)
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

/// Extract raw API key from X-API-Key header or ?token=/?api_key= query param.
fn extract_key(request: &Request) -> Option<String> {
    // Try X-API-Key header
    if let Some(val) = request.headers().get("X-API-Key") {
        if let Ok(s) = val.to_str() {
            if !s.is_empty() {
                return Some(s.to_string());
            }
        }
    }

    // Try query param
    let uri = request.uri();
    uri.query().and_then(|q| {
        url::form_urlencoded::parse(q.as_bytes())
            .find(|(k, _)| k == "token" || k == "api_key")
            .map(|(_, v)| v.to_string())
    })
}

/// Extract JWT from Authorization: Bearer <token> header.
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
    None
}

/// Auth context injected into request extensions after successful auth.
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: uuid::Uuid,
    pub key_id: uuid::Uuid,
    pub plan: String,
    pub is_admin: bool,
}

pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::any())
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
        ]);

    Router::new()
        // Public
        .route("/health", get(health))
        .route("/", get(root))
        .route("/api/v1/auth/register", post(super::auth::register))
        .route("/api/v1/auth/login", post(super::auth::login))
        .route("/api/v1/auth/verify", post(super::auth::verify_email))
        .route("/api/v1/plans", get(super::plans::list_plans))
        // OAuth (public)
        .route(
            "/api/v1/auth/oauth/{provider}/url",
            get(super::auth::oauth_url),
        )
        .route(
            "/api/v1/auth/oauth/{provider}/callback",
            post(super::auth::oauth_callback),
        )
        // Authenticated
        .route("/api/v1/auth/me", get(super::auth::me))
        .route("/api/v1/keys", get(super::keys::list_keys))
        .route("/api/v1/keys", post(super::keys::create_key))
        .route("/api/v1/keys/{id}", delete(super::keys::revoke_key))
        .route("/api/v1/keys/{id}", patch(super::keys::update_key))
        .route(
            "/api/v1/config",
            get(super::tenant_config::list_config),
        )
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
        // Admin routes
        .route("/api/v1/admin/users", get(super::admin::list_users))
        .route("/api/v1/admin/users/:id/plan", post(super::admin::set_user_plan))
        .route("/api/v1/admin/users/:id/toggle", post(super::admin::toggle_user))
        .route("/api/v1/admin/stats", get(super::admin::platform_stats))
        // Middleware
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
