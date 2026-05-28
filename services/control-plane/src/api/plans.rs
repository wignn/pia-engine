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

/// POST /api/v1/plans/upgrade
pub async fn upgrade(
    State(_state): State<AppState>,
    request: axum::extract::Request,
) -> Result<Json<Value>, StatusCode> {
    let _auth = request
        .extensions()
        .get::<AuthContext>()
        .cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;
    Ok(Json(json!({
        "status": "not_available",
        "message": "Plan upgrades via payment coming soon. Contact admin for manual upgrades.",
    })))
}

#[cfg(test)]
mod tests {
    use super::normalize_ws_connections;

    #[test]
    fn normalize_ws_connections_accepts_safe_admin_limits() {
        assert_eq!(normalize_ws_connections(1).unwrap(), 1);
        assert_eq!(normalize_ws_connections(1000).unwrap(), 1000);
        assert!(normalize_ws_connections(0).is_err());
        assert!(normalize_ws_connections(1001).is_err());
    }
}
