use axum::{extract::State, http::StatusCode, Json};
use serde_json::{json, Value};

use crate::api::server::AuthContext;
use crate::api::AppState;
use crate::models::plan::Plan;

/// GET /api/v1/plans
pub async fn list_plans(
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let plans = Plan::list_active(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json!({ "plans": plans })))
}

/// POST /api/v1/plans/upgrade — STUBBED
pub async fn upgrade(
    State(_state): State<AppState>,
    request: axum::extract::Request,
) -> Result<Json<Value>, StatusCode> {
    let _auth = request.extensions().get::<AuthContext>().cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;
    Ok(Json(json!({
        "status": "not_available",
        "message": "Plan upgrades via payment coming soon. Contact admin for manual upgrades.",
    })))
}
