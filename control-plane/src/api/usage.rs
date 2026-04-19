use axum::{extract::{Query, State}, http::StatusCode, Json};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::api::server::AuthContext;
use crate::api::AppState;
use crate::models::plan::Plan;
use crate::models::usage::UsageLog;

/// GET /api/v1/usage
pub async fn summary(
    State(state): State<AppState>,
    request: axum::extract::Request,
) -> Result<Json<Value>, StatusCode> {
    let auth = request.extensions().get::<AuthContext>().cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let (today, week, month) = UsageLog::summary(&state.db, auth.user_id)
        .await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let plan = Plan::find_by_id(&state.db, &auth.plan).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let limit = plan.map(|p| p.requests_per_day).unwrap_or(100);
    Ok(Json(json!({
        "today": today, "this_week": week, "this_month": month,
        "daily_limit": limit,
        "remaining_today": (limit as i64 - today).max(0),
    })))
}

#[derive(Deserialize)]
pub struct HistoryQuery { pub days: Option<i32> }

/// GET /api/v1/usage/history
pub async fn history(
    State(state): State<AppState>,
    Query(q): Query<HistoryQuery>,
    request: axum::extract::Request,
) -> Result<Json<Value>, StatusCode> {
    let auth = request.extensions().get::<AuthContext>().cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let days = q.days.unwrap_or(30).min(90);
    let data = UsageLog::daily_history(&state.db, auth.user_id, days)
        .await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json!({ "history": data, "days": days })))
}
