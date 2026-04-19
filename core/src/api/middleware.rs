use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::api::state::AppState;
use crate::tenant::context::TenantContext;

/// API key authentication middleware.
/// Validates keys against DB-backed tenant registry (cached in memory).
/// Falls back to env-based API_KEYS for admin/backward compatibility.
/// Injects TenantContext into request extensions on success.
pub async fn api_key_auth(
    axum::extract::State(state): axum::extract::State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = request.uri().path();
    let method = request.method().clone();

    // Skip auth for public endpoints
    if path == "/health" || path == "/" {
        return Ok(next.run(request).await);
    }
    // WebSocket auth is handled at upgrade time (via query param)
    if path.starts_with("/api/v1/ws/") {
        return Ok(next.run(request).await);
    }
    // Public GET endpoints
    if method == axum::http::Method::GET
        && (path.starts_with("/api/v1/forex/news")
            || path.starts_with("/api/v1/equity/news"))
    {
        return Ok(next.run(request).await);
    }

    // Extract API key from header, bearer, or query param
    let raw_key = extract_key(&request);

    let Some(raw) = raw_key else {
        // No API keys configured at all = allow (backward compat)
        if state.config.api_keys.is_empty() {
            tracing::warn!("no API keys configured, all requests allowed");
            return Ok(next.run(request).await);
        }
        return Err(StatusCode::UNAUTHORIZED);
    };

    // 1. Check env-based admin keys (backward compatibility for "olin" etc.)
    if state.config.api_keys.contains(&raw) {
        request.extensions_mut().insert(TenantContext::admin());
        return Ok(next.run(request).await);
    }

    // 2. Check DB-backed tenant registry
    if let Some(registry) = &state.tenant_registry {
        if let Some(ctx) = registry.validate_key(&raw).await {
            request.extensions_mut().insert(ctx);
            return Ok(next.run(request).await);
        }
    }

    tracing::warn!(
        path = %request.uri().path(),
        key_prefix = %if raw.len() > 16 { &raw[..16] } else { &raw },
        "API key auth failed — key not found in env or tenant registry"
    );

    Err(StatusCode::UNAUTHORIZED)
}

fn extract_key(request: &Request) -> Option<String> {
    // X-API-Key header
    if let Some(val) = request.headers().get("X-API-Key") {
        if let Ok(s) = val.to_str() {
            if !s.is_empty() {
                return Some(s.to_string());
            }
        }
    }

    // Authorization: Bearer <token>
    if let Some(val) = request.headers().get(header::AUTHORIZATION) {
        if let Ok(s) = val.to_str() {
            if let Some(token) = s.strip_prefix("Bearer ") {
                if !token.is_empty() {
                    return Some(token.to_string());
                }
            }
        }
    }

    // Query param: ?api_key= or ?token=
    let uri = request.uri();
    uri.query().and_then(|q| {
        url::form_urlencoded::parse(q.as_bytes())
            .find(|(k, _)| k == "api_key" || k == "token")
            .map(|(_, v)| v.to_string())
    })
}
