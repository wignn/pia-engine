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

pub async fn optional_api_key_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if let Some(raw_key) = extract_api_key(&request) {
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
