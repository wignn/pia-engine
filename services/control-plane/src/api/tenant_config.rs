use std::collections::HashSet;

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

pub async fn list_config(
    State(state): State<AppState>,
    request: axum::extract::Request,
) -> Result<Json<Value>, StatusCode> {
    let auth = request
        .extensions()
        .get::<AuthContext>()
        .cloned()
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

pub async fn set_config(
    State(state): State<AppState>,
    Path(config_key): Path<String>,
    request: axum::extract::Request,
) -> Result<Json<Value>, StatusCode> {
    let auth = request
        .extensions()
        .get::<AuthContext>()
        .cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let allowed_keys = ["tv_symbols", "custom_rss_feeds", "x_usernames"];
    if !allowed_keys.contains(&config_key.as_str()) {
        return Err(StatusCode::BAD_REQUEST);
    }

    if !auth.is_admin && (config_key == "tv_symbols" || config_key == "custom_rss_feeds") {
        return Err(StatusCode::FORBIDDEN);
    }

    let body_bytes = axum::body::to_bytes(request.into_body(), 1024 * 64)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let body: SetConfigRequest =
        serde_json::from_slice(&body_bytes).map_err(|_| StatusCode::BAD_REQUEST)?;

    let plan = Plan::find_by_id(&state.db, &auth.plan)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let value = match normalize_config_value(&config_key, body.value, &plan) {
        Ok(value) => value,
        Err(msg) => return Ok(Json(json!({ "error": msg }))),
    };

    let config = TenantConfig::set(&state.db, auth.user_id, &config_key, &value)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!(user_id = %auth.user_id, key = %config_key, "tenant config updated");

    sync::publish_config_changed_for_user(
        &state.redis,
        &state.config.redis_channel_prefix,
        Some(auth.user_id),
    )
    .await;

    Ok(Json(json!({
        "config": {
            "key": config.config_key,
            "value": config.config_value,
            "updated_at": config.updated_at,
        },
        "message": "Configuration updated successfully"
    })))
}

pub async fn delete_config(
    State(state): State<AppState>,
    Path(config_key): Path<String>,
    request: axum::extract::Request,
) -> Result<Json<Value>, StatusCode> {
    let auth = request
        .extensions()
        .get::<AuthContext>()
        .cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !auth.is_admin && (config_key == "tv_symbols" || config_key == "custom_rss_feeds") {
        return Err(StatusCode::FORBIDDEN);
    }

    let deleted = TenantConfig::delete(&state.db, auth.user_id, &config_key)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if deleted {
        sync::publish_config_changed_for_user(
            &state.redis,
            &state.config.redis_channel_prefix,
            Some(auth.user_id),
        )
        .await;
        Ok(Json(
            json!({ "message": format!("Config '{}' deleted", config_key) }),
        ))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

fn normalize_config_value(key: &str, value: Value, plan: &Plan) -> Result<Value, String> {
    match key {
        "tv_symbols" => {
            normalize_limited_string_array(value, plan.tv_symbols_max, "TV symbols", true)
        }
        "x_usernames" => {
            normalize_limited_string_array(value, plan.x_usernames_max, "X usernames", false)
        }
        "custom_rss_feeds" => {
            if !plan.can_custom_rss {
                return Err("Custom RSS feeds require Pro plan or higher".into());
            }
            Ok(value)
        }
        _ => Ok(value),
    }
}

fn normalize_limited_string_array(
    value: Value,
    max_items: i32,
    label: &str,
    uppercase: bool,
) -> Result<Value, String> {
    let arr = value
        .as_array()
        .ok_or_else(|| format!("{} must be an array of strings", label))?;

    let mut seen = HashSet::new();
    let mut normalized = Vec::new();

    for item in arr {
        let raw = item
            .as_str()
            .ok_or_else(|| format!("Each {} item must be a non-empty string", label))?;
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(format!("Each {} item must be a non-empty string", label));
        }

        let value = if uppercase {
            trimmed.to_uppercase()
        } else {
            trimmed.to_string()
        };

        if seen.insert(value.clone()) {
            normalized.push(Value::String(value));
        }
    }

    if normalized.len() > max_items as usize {
        return Err(format!(
            "Your plan allows max {} {}, got {}. Upgrade your plan for more.",
            max_items,
            label,
            normalized.len()
        ));
    }

    Ok(Value::Array(normalized))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan(tv_symbols_max: i32, x_usernames_max: i32, can_custom_rss: bool) -> Plan {
        Plan {
            id: "free".into(),
            name: "Free".into(),
            price_idr: 0,
            requests_per_day: 100,
            ws_connections: 1,
            x_usernames_max,
            tv_symbols_max,
            news_history_days: 1,
            rate_limit_per_min: 10,
            can_scrape: false,
            can_custom_rss,
            is_active: true,
            sort_order: 0,
        }
    }

    #[test]
    fn tv_symbols_are_trimmed_uppercased_and_deduped() {
        let value = json!([" eurusd ", "GBPUSD", "eurusd"]);
        let normalized = normalize_config_value("tv_symbols", value, &plan(3, 1, false)).unwrap();

        assert_eq!(normalized, json!(["EURUSD", "GBPUSD"]));
    }

    #[test]
    fn tv_symbols_reject_values_over_plan_limit_after_dedupe() {
        let value = json!(["EURUSD", "GBPUSD", "USDJPY", "AUDUSD"]);

        assert!(normalize_config_value("tv_symbols", value, &plan(3, 1, false)).is_err());
    }

    #[test]
    fn tv_symbols_reject_empty_strings() {
        let value = json!(["EURUSD", " "]);

        assert!(normalize_config_value("tv_symbols", value, &plan(3, 1, false)).is_err());
    }

    #[test]
    fn x_usernames_are_limited_without_uppercasing() {
        let value = json!([" wignn ", "market_bot"]);
        let normalized = normalize_config_value("x_usernames", value, &plan(3, 2, false)).unwrap();

        assert_eq!(normalized, json!(["wignn", "market_bot"]));
    }

    #[test]
    fn custom_rss_requires_plan_permission() {
        let value = json!(["https://example.com/feed.xml"]);

        assert!(normalize_config_value("custom_rss_feeds", value, &plan(3, 1, false)).is_err());
    }
}
