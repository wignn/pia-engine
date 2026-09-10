use axum::{extract::State, http::StatusCode, Json};
use serde_json::{json, Value};

use crate::api::server::AuthContext;
use crate::api::AppState;
use crate::models::plan::Plan;

/// GET /api/v1/plans
pub async fn list_plans(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    let plans = Plan::list_active(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json!({ "plans": plans })))
}

#[derive(serde::Deserialize)]
pub struct UpdatePlanWsConnectionsRequest {
    pub ws_connections: i32,
}

pub async fn update_plan_ws_connections(
    State(state): State<AppState>,
    axum::extract::Path(plan_id): axum::extract::Path<String>,
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
    let body: UpdatePlanWsConnectionsRequest =
        serde_json::from_slice(&body_bytes).map_err(|_| StatusCode::BAD_REQUEST)?;
    let ws_connections = normalize_ws_connections(body.ws_connections)?;

    let updated = Plan::update_ws_connections(&state.db, &plan_id, ws_connections)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if !updated {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(Json(json!({
        "message": "Plan WebSocket limit updated",
        "plan": plan_id,
        "ws_connections": ws_connections,
    })))
}

fn normalize_ws_connections(value: i32) -> Result<i32, StatusCode> {
    if !(1..=1000).contains(&value) {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(value)
}

#[derive(serde::Deserialize)]
pub struct UpgradePlanRequest {
    pub plan_id: Option<String>,
    pub plan: Option<String>,
}

/// POST /api/v1/plans/upgrade
pub async fn upgrade(
    State(state): State<AppState>,
    request: axum::extract::Request,
) -> Result<Json<Value>, StatusCode> {
    let auth = request
        .extensions()
        .get::<AuthContext>()
        .cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let body_bytes = axum::body::to_bytes(request.into_body(), 1024)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let body: UpgradePlanRequest =
        serde_json::from_slice(&body_bytes).map_err(|_| StatusCode::BAD_REQUEST)?;

    let target_plan = body.plan_id.or(body.plan).ok_or(StatusCode::BAD_REQUEST)?;
    let target_plan = target_plan.trim().to_lowercase();

    let plan_exists = Plan::find_by_id(&state.db, &target_plan)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let Some(plan) = plan_exists else {
        return Ok(Json(json!({
            "error": format!("Plan '{}' does not exist", target_plan)
        })));
    };

    if !plan.is_active {
        return Ok(Json(json!({
            "error": format!("Plan '{}' is not currently active", target_plan)
        })));
    }

    sqlx::query("UPDATE users SET plan = $1, updated_at = NOW() WHERE id = $2")
        .bind(&target_plan)
        .bind(auth.user_id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    crate::sync::publish_config_changed_for_user(
        &state.redis,
        &state.config.redis_channel_prefix,
        Some(auth.user_id),
    )
    .await;

    tracing::info!(user_id = %auth.user_id, plan = %target_plan, "user upgraded plan");

    Ok(Json(json!({
        "status": "active",
        "plan": target_plan,
        "message": format!("Successfully switched to {} plan", plan.name),
        "limits": plan
    })))
}

#[cfg(test)]
mod tests {
    use super::{normalize_ws_connections, UpgradePlanRequest};

    #[test]
    fn normalize_ws_connections_accepts_safe_admin_limits() {
        assert_eq!(normalize_ws_connections(1).unwrap(), 1);
        assert_eq!(normalize_ws_connections(1000).unwrap(), 1000);
        assert!(normalize_ws_connections(0).is_err());
        assert!(normalize_ws_connections(1001).is_err());
    }

    #[test]
    fn upgrade_plan_request_accepts_both_plan_and_plan_id() {
        let json_plan_id = r#"{"plan_id":"pro"}"#;
        let req1: UpgradePlanRequest = serde_json::from_str(json_plan_id).unwrap();
        assert_eq!(req1.plan_id.as_deref(), Some("pro"));

        let json_plan = r#"{"plan":"starter"}"#;
        let req2: UpgradePlanRequest = serde_json::from_str(json_plan).unwrap();
        assert_eq!(req2.plan.as_deref(), Some("starter"));
    }
}
