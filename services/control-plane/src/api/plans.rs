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

    if target_plan == auth.plan {
        return Ok(Json(json!({
            "error": format!("You are already subscribed to the {} plan", plan.name)
        })));
    }

    let existing_request: Option<(uuid::Uuid,)> = sqlx::query_as(
        "SELECT id FROM plan_change_requests WHERE user_id = $1 AND status = 'pending'",
    )
    .bind(auth.user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some((req_id,)) = existing_request {
        sqlx::query(
            "UPDATE plan_change_requests SET requested_plan = $1, created_at = NOW() WHERE id = $2",
        )
        .bind(&target_plan)
        .bind(req_id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    } else {
        sqlx::query(
            "INSERT INTO plan_change_requests (user_id, current_plan, requested_plan, status) VALUES ($1, $2, $3, 'pending')",
        )
        .bind(auth.user_id)
        .bind(&auth.plan)
        .bind(&target_plan)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    tracing::info!(user_id = %auth.user_id, current_plan = %auth.plan, requested_plan = %target_plan, "user requested plan change");

    Ok(Json(json!({
        "status": "pending",
        "requested_plan": target_plan,
        "message": format!("Plan change request for {} submitted. Awaiting administrator approval.", plan.name),
        "limits": plan
    })))
}

/// GET /api/v1/plans/request — check if user has a pending plan request
pub async fn current_request(
    State(state): State<AppState>,
    request: axum::extract::Request,
) -> Result<Json<Value>, StatusCode> {
    let auth = request
        .extensions()
        .get::<AuthContext>()
        .cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let row: Option<(
        uuid::Uuid,
        String,
        String,
        String,
        chrono::DateTime<chrono::Utc>,
    )> = sqlx::query_as(
        "SELECT id, current_plan, requested_plan, status, created_at \
         FROM plan_change_requests \
         WHERE user_id = $1 AND status = 'pending' \
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(auth.user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some((id, cur, req, status, created_at)) = row {
        Ok(Json(json!({
            "has_pending": true,
            "request": {
                "id": id,
                "current_plan": cur,
                "requested_plan": req,
                "status": status,
                "created_at": created_at
            }
        })))
    } else {
        Ok(Json(json!({ "has_pending": false, "request": null })))
    }
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
