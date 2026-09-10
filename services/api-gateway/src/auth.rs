use atlsd_domain::tenant::TenantContext;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::time::Instant;

use crate::state::AppState;
use crate::usage::{RateLimitDecision, UsageEvent};

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
        let req_scope = required_scope_for_path(request.uri().path());
        if !ctx.has_permission(req_scope) {
            let res = (
                StatusCode::FORBIDDEN,
                axum::Json(serde_json::json!({
                    "error": "insufficient_scope",
                    "message": format!("API key does not have the required '{}' scope permission.", req_scope),
                    "required_scope": req_scope
                })),
            )
                .into_response();
            return Ok(res);
        }

        let decision = state.usage_tracker.check_rate_limit(&ctx).await;
        match decision {
            RateLimitDecision::Allowed {
                min_limit,
                min_remaining,
                day_limit,
                day_remaining,
                reset_seconds,
            } => {
                request.extensions_mut().insert(ctx);
                let mut response = next.run(request).await;
                let headers = response.headers_mut();
                headers.insert(
                    "x-ratelimit-limit",
                    axum::http::HeaderValue::from(min_limit),
                );
                headers.insert(
                    "x-ratelimit-remaining",
                    axum::http::HeaderValue::from(min_remaining),
                );
                headers.insert(
                    "x-ratelimit-reset",
                    axum::http::HeaderValue::from(reset_seconds),
                );
                headers.insert(
                    "x-dailyquota-limit",
                    axum::http::HeaderValue::from(day_limit),
                );
                headers.insert(
                    "x-dailyquota-remaining",
                    axum::http::HeaderValue::from(day_remaining),
                );
                Ok(response)
            }
            RateLimitDecision::MinuteExceeded {
                limit,
                reset_seconds,
            } => {
                let mut res = (
                    StatusCode::TOO_MANY_REQUESTS,
                    axum::Json(serde_json::json!({
                        "error": "rate_limit_exceeded",
                        "message": format!("Burst rate limit of {} req/min exceeded. Please retry after {} seconds.", limit, reset_seconds),
                        "limit": limit,
                        "retry_after_seconds": reset_seconds,
                        "upgrade_url": "https://pia.wign.dev/portal/account"
                    })),
                )
                    .into_response();
                let headers = res.headers_mut();
                headers.insert("retry-after", axum::http::HeaderValue::from(reset_seconds));
                headers.insert(
                    "x-ratelimit-reset",
                    axum::http::HeaderValue::from(reset_seconds),
                );
                headers.insert("x-ratelimit-limit", axum::http::HeaderValue::from(limit));
                headers.insert(
                    "x-ratelimit-remaining",
                    axum::http::HeaderValue::from_static("0"),
                );
                Ok(res)
            }
            RateLimitDecision::DailyQuotaExceeded { limit } => {
                let mut res = (
                    StatusCode::TOO_MANY_REQUESTS,
                    axum::Json(serde_json::json!({
                        "error": "daily_quota_exceeded",
                        "message": format!("Daily quota of {} requests exceeded for your tier. Upgrade your plan to continue.", limit),
                        "limit": limit,
                        "upgrade_url": "https://pia.wign.dev/portal/account"
                    })),
                )
                    .into_response();
                let headers = res.headers_mut();
                headers.insert("x-dailyquota-limit", axum::http::HeaderValue::from(limit));
                headers.insert(
                    "x-dailyquota-remaining",
                    axum::http::HeaderValue::from_static("0"),
                );
                Ok(res)
            }
            RateLimitDecision::Bypassed => {
                request.extensions_mut().insert(ctx);
                Ok(next.run(request).await)
            }
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
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

pub fn required_scope_for_path(path: &str) -> &'static str {
    if path.starts_with("/api/v1/social") {
        "social:read"
    } else if path.starts_with("/api/v1/news") || path.starts_with("/api/v1/forex/news") {
        "news:read"
    } else if path.starts_with("/api/v1/market/economic")
        || path.starts_with("/api/v1/central-banks")
        || path.starts_with("/api/v1/geosignals")
    {
        "macro:read"
    } else {
        "market:read"
    }
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

    #[test]
    fn rate_limit_decision_is_allowed_logic() {
        let allowed = RateLimitDecision::Allowed {
            min_limit: 60,
            min_remaining: 59,
            day_limit: 5000,
            day_remaining: 4999,
            reset_seconds: 60,
        };
        assert!(allowed.is_allowed());

        let bypassed = RateLimitDecision::Bypassed;
        assert!(bypassed.is_allowed());

        let min_exceeded = RateLimitDecision::MinuteExceeded {
            limit: 60,
            reset_seconds: 30,
        };
        assert!(!min_exceeded.is_allowed());

        let day_exceeded = RateLimitDecision::DailyQuotaExceeded { limit: 5000 };
        assert!(!day_exceeded.is_allowed());
    }

    #[test]
    fn required_scope_for_path_mapping() {
        assert_eq!(
            required_scope_for_path("/api/v1/market/prices"),
            "market:read"
        );
        assert_eq!(
            required_scope_for_path("/api/v1/options/summary"),
            "market:read"
        );
        assert_eq!(
            required_scope_for_path("/api/v1/social/posts"),
            "social:read"
        );
        assert_eq!(required_scope_for_path("/api/v1/news/feed"), "news:read");
        assert_eq!(
            required_scope_for_path("/api/v1/market/economic/indicators"),
            "macro:read"
        );
        assert_eq!(
            required_scope_for_path("/api/v1/central-banks/fed/stance"),
            "macro:read"
        );
        assert_eq!(
            required_scope_for_path("/api/v1/geosignals/map"),
            "macro:read"
        );
    }
}
