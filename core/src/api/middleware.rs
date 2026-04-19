use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use std::time::Instant;

use crate::api::state::AppState;
use crate::api::usage_tracker::UsageEvent;
use crate::tenant::context::TenantContext;

pub async fn optional_api_key_auth(
    axum::extract::State(state): axum::extract::State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let _ = attach_tenant_context_if_valid(&state, &mut request).await;

    if let Some(ctx) = request.extensions().get::<TenantContext>() {
        if !state.usage_tracker.try_consume_daily_quota(ctx).await {
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }
    }

    Ok(next.run(request).await)
}

pub async fn strict_api_key_auth(
    axum::extract::State(state): axum::extract::State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if !attach_tenant_context_if_valid(&state, &mut request).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    if let Some(ctx) = request.extensions().get::<TenantContext>() {
        if !state.usage_tracker.try_consume_daily_quota(ctx).await {
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }
    }

    Ok(next.run(request).await)
}

pub async fn usage_logger(
    axum::extract::State(state): axum::extract::State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = request.uri().path().to_string();
    let method = request.method().clone();
    let usage_ctx = request.extensions().get::<TenantContext>().cloned();
    let started = Instant::now();
    let response = next.run(request).await;

    if let Some(ctx) = usage_ctx {
        if !ctx.is_admin {
            let elapsed_ms = started.elapsed().as_millis().min(i32::MAX as u128) as i32;
            state
                .usage_tracker
                .enqueue(UsageEvent {
                    user_id: ctx.user_id,
                    api_key_id: ctx.api_key_id,
                    endpoint: path,
                    method: method.as_str().to_string(),
                    status_code: i32::from(response.status().as_u16()),
                    response_ms: elapsed_ms,
                })
                .await;
        }
    }

    Ok(response)
}

async fn attach_tenant_context_if_valid(state: &AppState, request: &mut Request) -> bool {
    let raw_key = extract_key(&request);

    match raw_key {
        Some(raw) => {
            if state.config.api_keys.contains(&raw) {
                request.extensions_mut().insert(TenantContext::admin());
                return true;
            } else if let Some(registry) = &state.tenant_registry {
                if let Some(ctx) = registry.validate_key(&raw).await {
                    request.extensions_mut().insert(ctx);
                    return true;
                }
                tracing::warn!(
                    path = %request.uri().path(),
                    key_prefix = %if raw.len() > 16 { &raw[..16] } else { &raw },
                    "API key auth failed — key not found in env or tenant registry"
                );
            }
        }
        None => {
            if state.config.api_keys.is_empty() {
                tracing::warn!("no API keys configured, all requests allowed");
                return true;
            }
        }
    }

    false
}

fn extract_key(request: &Request) -> Option<String> {
    if let Some(val) = request.headers().get("X-API-Key") {
        if let Ok(s) = val.to_str() {
            if !s.is_empty() {
                return Some(s.to_string());
            }
        }
    }

    if let Some(val) = request.headers().get(header::AUTHORIZATION) {
        if let Ok(s) = val.to_str() {
            if let Some(token) = s.strip_prefix("Bearer ") {
                if !token.is_empty() {
                    return Some(token.to_string());
                }
            }
        }
    }

    let uri = request.uri();
    uri.query().and_then(|q| {
        url::form_urlencoded::parse(q.as_bytes())
            .find(|(k, _)| k == "api_key" || k == "token")
            .map(|(_, v)| v.to_string())
    })
}
