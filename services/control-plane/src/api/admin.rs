use axum::{extract::State, http::StatusCode, Json};
use serde_json::{json, Value};

use crate::api::server::AuthContext;
use crate::api::AppState;
use crate::models::api_key::{ApiKey, ApiKeyInfo};
use crate::sync;

/// GET /api/v1/admin/users — list all users (admin only)
pub async fn list_users(
    State(state): State<AppState>,
    request: axum::extract::Request,
) -> Result<Json<Value>, StatusCode> {
    let auth = request
        .extensions()
        .get::<AuthContext>()
        .cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !auth.is_admin {
        return Err(StatusCode::FORBIDDEN);
    }

    type UserRow = (
        uuid::Uuid,
        String,
        String,
        String,
        bool,
        bool,
        chrono::DateTime<chrono::Utc>,
    );
    let rows: Vec<UserRow> = sqlx::query_as(
        "SELECT u.id, u.email, u.name, u.plan, u.is_active, u.email_verified, u.created_at \
             FROM users u \
             ORDER BY u.created_at DESC \
             LIMIT 500",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Key counts per user
    let key_counts: Vec<(uuid::Uuid, i64)> = sqlx::query_as(
        "SELECT user_id, COUNT(*) FROM api_keys WHERE is_active = TRUE GROUP BY user_id",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let key_map: std::collections::HashMap<uuid::Uuid, i64> = key_counts.into_iter().collect();

    let users: Vec<Value> = rows
        .into_iter()
        .map(
            |(id, email, name, plan, is_active, email_verified, created_at)| {
                json!({
                    "id": id,
                    "email": email,
                    "name": name,
                    "plan": plan,
                    "is_active": is_active,
                    "email_verified": email_verified,
                    "created_at": created_at,
                    "active_keys": key_map.get(&id).copied().unwrap_or(0),
                })
            },
        )
        .collect();

    Ok(Json(json!({
        "users": users,
        "total": users.len(),
    })))
}

/// GET /api/v1/admin/users/:id/keys — list a user's API keys (admin only)
pub async fn list_user_keys(
    State(state): State<AppState>,
    axum::extract::Path(user_id): axum::extract::Path<uuid::Uuid>,
    request: axum::extract::Request,
) -> Result<Json<Value>, StatusCode> {
    let auth = request
        .extensions()
        .get::<AuthContext>()
        .cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !auth.is_admin {
        return Err(StatusCode::FORBIDDEN);
    }

    let keys = ApiKey::list_by_user(&state.db, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let infos: Vec<ApiKeyInfo> = keys.into_iter().map(ApiKeyInfo::from).collect();

    Ok(Json(json!({
        "keys": infos,
        "total": infos.len(),
    })))
}

/// POST /api/v1/admin/users/:id/plan — update user plan (admin only)
pub async fn set_user_plan(
    State(state): State<AppState>,
    axum::extract::Path(user_id): axum::extract::Path<uuid::Uuid>,
    request: axum::extract::Request,
) -> Result<Json<Value>, StatusCode> {
    let auth = request
        .extensions()
        .get::<AuthContext>()
        .cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !auth.is_admin {
        return Err(StatusCode::FORBIDDEN);
    }

    let body_bytes = axum::body::to_bytes(request.into_body(), 1024)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let body: Value = serde_json::from_slice(&body_bytes).map_err(|_| StatusCode::BAD_REQUEST)?;

    let plan = body["plan"].as_str().ok_or(StatusCode::BAD_REQUEST)?;

    let valid_plans = ["free", "starter", "pro", "enterprise"];
    if !valid_plans.contains(&plan) {
        return Ok(Json(
            json!({ "error": "Invalid plan. Must be one of: free, starter, pro, enterprise" }),
        ));
    }

    sqlx::query("UPDATE users SET plan = $1, updated_at = NOW() WHERE id = $2")
        .bind(plan)
        .bind(user_id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    sync::publish_config_changed_for_user(
        &state.redis,
        &state.config.redis_channel_prefix,
        Some(user_id),
    )
    .await;

    Ok(Json(
        json!({ "message": format!("User plan updated to {}", plan), "plan": plan }),
    ))
}

/// POST /api/v1/admin/users/:id/toggle — activate/deactivate user (admin only)
pub async fn toggle_user(
    State(state): State<AppState>,
    axum::extract::Path(user_id): axum::extract::Path<uuid::Uuid>,
    request: axum::extract::Request,
) -> Result<Json<Value>, StatusCode> {
    let auth = request
        .extensions()
        .get::<AuthContext>()
        .cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !auth.is_admin {
        return Err(StatusCode::FORBIDDEN);
    }

    let row: Option<(bool,)> = sqlx::query_as("SELECT is_active FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let Some((current,)) = row else {
        return Ok(Json(json!({ "error": "User not found" })));
    };

    let new_status = !current;
    sqlx::query("UPDATE users SET is_active = $1, updated_at = NOW() WHERE id = $2")
        .bind(new_status)
        .bind(user_id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    sync::publish_config_changed_for_user(
        &state.redis,
        &state.config.redis_channel_prefix,
        Some(user_id),
    )
    .await;

    Ok(Json(json!({
        "message": if new_status { "User activated" } else { "User deactivated" },
        "is_active": new_status,
    })))
}

/// GET /api/v1/admin/stats — platform stats (admin only)
pub async fn platform_stats(
    State(state): State<AppState>,
    request: axum::extract::Request,
) -> Result<Json<Value>, StatusCode> {
    let auth = request
        .extensions()
        .get::<AuthContext>()
        .cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !auth.is_admin {
        return Err(StatusCode::FORBIDDEN);
    }

    let (total_users,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db)
        .await
        .unwrap_or((0,));

    let (active_users,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM users WHERE is_active = TRUE")
            .fetch_one(&state.db)
            .await
            .unwrap_or((0,));

    let (total_keys,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM api_keys WHERE is_active = TRUE")
            .fetch_one(&state.db)
            .await
            .unwrap_or((0,));

    let plan_counts: Vec<(String, i64)> =
        sqlx::query_as("SELECT plan, COUNT(*) FROM users GROUP BY plan ORDER BY COUNT(*) DESC")
            .fetch_all(&state.db)
            .await
            .unwrap_or_default();

    let plans_map: serde_json::Map<String, Value> = plan_counts
        .into_iter()
        .map(|(plan, count)| (plan, json!(count)))
        .collect();

    Ok(Json(json!({
        "total_users": total_users,
        "active_users": active_users,
        "total_api_keys": total_keys,
        "users_by_plan": plans_map,
    })))
}

/// GET /api/v1/admin/users/:id/usage — get a user's usage details (admin only)
pub async fn user_usage(
    State(state): State<AppState>,
    axum::extract::Path(user_id): axum::extract::Path<uuid::Uuid>,
    request: axum::extract::Request,
) -> Result<Json<Value>, StatusCode> {
    let auth = request
        .extensions()
        .get::<AuthContext>()
        .cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !auth.is_admin {
        return Err(StatusCode::FORBIDDEN);
    }

    let (db_today, week, month) = crate::models::usage::UsageLog::summary(&state.db, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut redis_today = 0i64;
    let daily_key = format!(
        "usage:daily:{}:{}",
        user_id,
        chrono::Utc::now().format("%Y-%m-%d")
    );
    if let Some(ref client) = state.redis {
        if let Ok(mut conn) = client.get_multiplexed_tokio_connection().await {
            let val: Result<Option<i64>, _> =
                redis::AsyncCommands::get(&mut conn, &daily_key).await;
            if let Ok(Some(count)) = val {
                redis_today = count;
            }
        }
    }

    let today = redis_today.max(db_today);

    let user_row: Option<(String,)> = sqlx::query_as("SELECT plan FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let plan_id = user_row.map(|r| r.0).unwrap_or_else(|| "free".to_string());
    let plan = crate::models::plan::Plan::find_by_id(&state.db, &plan_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let limit = plan.map(|p| p.requests_per_day as i64).unwrap_or(100);

    Ok(Json(json!({
        "user_id": user_id,
        "plan": plan_id,
        "today": today,
        "this_week": week.max(today),
        "this_month": month.max(today),
        "daily_limit": limit,
        "remaining_today": (limit - today).max(0),
    })))
}

/// POST /api/v1/admin/users/:id/quota/reset — reset user's daily usage in Redis to 0 (admin only)
pub async fn reset_user_quota(
    State(state): State<AppState>,
    axum::extract::Path(user_id): axum::extract::Path<uuid::Uuid>,
    request: axum::extract::Request,
) -> Result<Json<Value>, StatusCode> {
    let auth = request
        .extensions()
        .get::<AuthContext>()
        .cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !auth.is_admin {
        return Err(StatusCode::FORBIDDEN);
    }

    let daily_key = format!(
        "usage:daily:{}:{}",
        user_id,
        chrono::Utc::now().format("%Y-%m-%d")
    );

    if let Some(ref client) = state.redis {
        if let Ok(mut conn) = client.get_multiplexed_tokio_connection().await {
            let _: Result<(), _> = redis::AsyncCommands::del(&mut conn, &daily_key).await;
        }
    }

    sync::publish_config_changed_for_user(
        &state.redis,
        &state.config.redis_channel_prefix,
        Some(user_id),
    )
    .await;

    Ok(Json(json!({
        "message": format!("Daily quota reset for user {}", user_id),
        "user_id": user_id,
        "today": 0
    })))
}

/// POST /api/v1/admin/system/flush-cache — broadcast sync and refresh all tenant caches (admin only)
pub async fn flush_cache(
    State(state): State<AppState>,
    request: axum::extract::Request,
) -> Result<Json<Value>, StatusCode> {
    let auth = request
        .extensions()
        .get::<AuthContext>()
        .cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !auth.is_admin {
        return Err(StatusCode::FORBIDDEN);
    }

    sync::publish_config_changed_for_user(&state.redis, &state.config.redis_channel_prefix, None)
        .await;

    Ok(Json(json!({
        "message": "Global config sync signal broadcasted to all gateways",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}
