use atlsd_domain::tenant::TenantContext;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::time::Instant;

use crate::state::AppState;
use crate::usage::UsageEvent;

pub async fn require_api_key_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let raw_key = require_api_key(&request)?;
    if is_admin_path(request.uri().path()) {
        if is_admin_key(&raw_key, &state.config.admin_api_key) {
            return Ok(next.run(request).await);
        }
        return Err(StatusCode::FORBIDDEN);
    }
    if state.config.api_keys.contains(&raw_key) {
        return Ok(next.run(request).await);
    }
    if let Some(ctx) = state.tenant_registry.validate_key(&raw_key).await {
        if !state.usage_tracker.try_consume_daily_quota(&ctx).await {
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }
        request.extensions_mut().insert(ctx);
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(next.run(request).await)
}

pub async fn usage_logger(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    let tenant = request.extensions().get::<TenantContext>().cloned();
    let response = next.run(request).await;
    if let Some(ctx) = tenant {
        state
            .usage_tracker
            .enqueue(UsageEvent {
                user_id: ctx.user_id,
                api_key_id: ctx.api_key_id,
                endpoint: path,
                method,
                status_code: response.status().as_u16() as i32,
                response_ms: start.elapsed().as_millis().min(i32::MAX as u128) as i32,
            })
            .await;
    }
    response
}

fn require_api_key(request: &Request) -> Result<String, StatusCode> {
    extract_api_key(request).ok_or(StatusCode::UNAUTHORIZED)
}

fn is_admin_path(path: &str) -> bool {
    path.starts_with("/api/v1/admin/")
}

fn is_admin_key(raw_key: &str, admin_api_key: &str) -> bool {
    !admin_api_key.trim().is_empty() && raw_key == admin_api_key
}

fn extract_api_key(request: &Request) -> Option<String> {
    request
        .headers()
        .get("X-API-Key")
        .or_else(|| request.headers().get(axum::http::header::AUTHORIZATION))
        .and_then(|h| h.to_str().ok())
        .map(|s| s.strip_prefix("Bearer ").unwrap_or(s).trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            request.uri().query().and_then(|query| {
                query.split('&').find_map(|pair| {
                    let (key, value) = pair.split_once('=')?;
                    (key == "api_key" || key == "token").then(|| value.to_string())
                })
            })
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;

    #[test]
    fn protected_requests_without_api_key_are_unauthorized() {
        let request = Request::builder()
            .uri("/api/v1/market/prices")
            .body(Body::empty())
            .unwrap();

        assert_eq!(require_api_key(&request), Err(StatusCode::UNAUTHORIZED));
    }

    #[test]
    fn protected_requests_accept_bearer_api_key() {
        let request = Request::builder()
            .uri("/api/v1/market/prices")
            .header(axum::http::header::AUTHORIZATION, "Bearer tenant-key")
            .body(Body::empty())
            .unwrap();

        assert_eq!(require_api_key(&request), Ok("tenant-key".to_string()));
    }

    #[test]
    fn admin_forex_paths_require_admin_key() {
        assert!(is_admin_path("/api/v1/admin/forex/sources"));
        assert!(is_admin_path(
            "/api/v1/admin/forex/sources/feed-fxstreet/toggle"
        ));
        assert!(!is_admin_path("/api/v1/forex/news"));
    }

    #[test]
    fn admin_key_must_match_configured_admin_key() {
        assert!(is_admin_key("admin-secret", "admin-secret"));
        assert!(!is_admin_key("tenant-key", "admin-secret"));
        assert!(!is_admin_key("admin-secret", ""));
    }
}
