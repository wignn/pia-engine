use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};
use tracing::info;

use crate::api::server::AuthContext;
use crate::api::AppState;
use crate::models::plan::Plan;
use crate::models::tenant_config::{SetConfigRequest, TenantConfig};
use crate::sync;

/// GET /api/v1/config
pub async fn list_config(
    State(state): State<AppState>,
    request: axum::extract::Request,
) -> Result<Json<Value>, StatusCode> {
    let auth = request.extensions().get::<AuthContext>().cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let configs = TenantConfig::list_by_user(&state.db, auth.user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let plan = Plan::find_by_id(&state.db, &auth.plan)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let config_map: serde_json::Map<String, Value> = configs
        .into_iter()
        .map(|c| (c.config_key, c.config_value))
        .collect();

    Ok(Json(json!({
        "configs": config_map,
        "plan_limits": plan,
    })))
}

/// PUT /api/v1/config/:key
pub async fn set_config(
    State(state): State<AppState>,
    Path(config_key): Path<String>,
    request: axum::extract::Request,
) -> Result<Json<Value>, StatusCode> {
    let auth = request.extensions().get::<AuthContext>().cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Validate config key
    let allowed_keys = ["x_usernames", "tv_symbols", "custom_rss_feeds"];
    if !allowed_keys.contains(&config_key.as_str()) {
        return Ok(Json(json!({ "error": format!("Unknown config key: {}. Allowed: {:?}", config_key, allowed_keys) })));
    }

    // We need to extract body - use a workaround since we already read extensions
    // In practice, the body would be parsed before extensions
    // For now, we'll create a placeholder
    let body_bytes = axum::body::to_bytes(request.into_body(), 1024 * 64)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let body: SetConfigRequest = serde_json::from_slice(&body_bytes)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Validate against plan limits
    let plan = Plan::find_by_id(&state.db, &auth.plan)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Err(msg) = validate_config(&config_key, &body.value, &plan) {
        return Ok(Json(json!({ "error": msg })));
    }

    let config = TenantConfig::set(&state.db, auth.user_id, &config_key, &body.value)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!(user_id = %auth.user_id, key = %config_key, "tenant config updated");

    // Notify core via Redis
    sync::publish_config_changed(&state.redis, &state.config.redis_channel_prefix).await;

    Ok(Json(json!({
        "config": {
            "key": config.config_key,
            "value": config.config_value,
            "updated_at": config.updated_at,
        },
        "message": "Configuration updated successfully"
    })))
}

/// DELETE /api/v1/config/:key
pub async fn delete_config(
    State(state): State<AppState>,
    Path(config_key): Path<String>,
    request: axum::extract::Request,
) -> Result<Json<Value>, StatusCode> {
    let auth = request.extensions().get::<AuthContext>().cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let deleted = TenantConfig::delete(&state.db, auth.user_id, &config_key)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if deleted {
        sync::publish_config_changed(&state.redis, &state.config.redis_channel_prefix).await;
        Ok(Json(json!({ "message": format!("Config '{}' deleted", config_key) })))
    } else {
        Ok(Json(json!({ "error": "Config not found" })))
    }
}

fn validate_config(key: &str, value: &Value, plan: &Plan) -> Result<(), String> {
    match key {
        "x_usernames" => {
            let arr = value.as_array().ok_or("x_usernames must be an array of strings")?;
            if arr.len() > plan.x_usernames_max as usize {
                return Err(format!(
                    "Your plan allows max {} X usernames, got {}. Upgrade your plan for more.",
                    plan.x_usernames_max,
                    arr.len()
                ));
            }
            for item in arr {
                if item.as_str().is_none() {
                    return Err("Each username must be a string".into());
                }
            }
            Ok(())
        }
        "tv_symbols" => {
            let arr = value.as_array().ok_or("tv_symbols must be an array of strings")?;
            if arr.len() > plan.tv_symbols_max as usize {
                return Err(format!(
                    "Your plan allows max {} TV symbols, got {}. Upgrade your plan for more.",
                    plan.tv_symbols_max,
                    arr.len()
                ));
            }
            Ok(())
        }
        "custom_rss_feeds" => {
            if !plan.can_custom_rss {
                return Err("Custom RSS feeds require Pro plan or higher".into());
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
