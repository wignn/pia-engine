use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};

use crate::api::server::AuthContext;
use crate::api::AppState;

/// GET /api/v1/admin/users — list all users (admin only)
pub async fn list_users(
    State(state): State<AppState>,
    request: axum::extract::Request,
) -> Result<Json<Value>, StatusCode> {
    let auth = request.extensions().get::<AuthContext>().cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !auth.is_admin {
        return Err(StatusCode::FORBIDDEN);
    }

    let rows: Vec<(uuid::Uuid, String, String, String, bool, bool, chrono::DateTime<chrono::Utc>)> =
        sqlx::query_as(
            "SELECT u.id, u.email, u.name, u.plan, u.is_active, u.email_verified, u.created_at \
             FROM users u \
             ORDER BY u.created_at DESC \
             LIMIT 500",
        )
        .fetch_all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Key counts per user
    let key_counts: Vec<(uuid::Uuid, i64)> =
        sqlx::query_as("SELECT user_id, COUNT(*) FROM api_keys WHERE is_active = TRUE GROUP BY user_id")
            .fetch_all(&state.db)
            .await
            .unwrap_or_default();

    let key_map: std::collections::HashMap<uuid::Uuid, i64> =
        key_counts.into_iter().collect();

    let users: Vec<Value> = rows
        .into_iter()
        .map(|(id, email, name, plan, is_active, email_verified, created_at)| {
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
        })
        .collect();

    Ok(Json(json!({
        "users": users,
        "total": users.len(),
    })))
}

/// POST /api/v1/admin/users/:id/plan — update user plan (admin only)
pub async fn set_user_plan(
    State(state): State<AppState>,
    axum::extract::Path(user_id): axum::extract::Path<uuid::Uuid>,
    request: axum::extract::Request,
) -> Result<Json<Value>, StatusCode> {
    let auth = request.extensions().get::<AuthContext>().cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !auth.is_admin {
        return Err(StatusCode::FORBIDDEN);
    }

    let body_bytes = axum::body::to_bytes(request.into_body(), 1024)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let body: Value = serde_json::from_slice(&body_bytes)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let plan = body["plan"].as_str().ok_or(StatusCode::BAD_REQUEST)?;

    let valid_plans = ["free", "basic", "pro", "enterprise"];
    if !valid_plans.contains(&plan) {
        return Ok(Json(json!({ "error": "Invalid plan. Must be one of: free, basic, pro, enterprise" })));
    }

    sqlx::query("UPDATE users SET plan = $1, updated_at = NOW() WHERE id = $2")
        .bind(plan)
        .bind(user_id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({ "message": format!("User plan updated to {}", plan) })))
}

/// POST /api/v1/admin/users/:id/toggle — activate/deactivate user (admin only)
pub async fn toggle_user(
    State(state): State<AppState>,
    axum::extract::Path(user_id): axum::extract::Path<uuid::Uuid>,
    request: axum::extract::Request,
) -> Result<Json<Value>, StatusCode> {
    let auth = request.extensions().get::<AuthContext>().cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !auth.is_admin {
        return Err(StatusCode::FORBIDDEN);
    }

    let row: Option<(bool,)> =
        sqlx::query_as("SELECT is_active FROM users WHERE id = $1")
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
    let auth = request.extensions().get::<AuthContext>().cloned()
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
