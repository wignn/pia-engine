use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
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
    let auth = request
        .extensions()
        .get::<AuthContext>()
        .cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let (db_today, week, month) = UsageLog::summary(&state.db, auth.user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut redis_today = 0i64;
    if let Some(ref client) = state.redis {
        if let Ok(mut conn) = client.get_multiplexed_tokio_connection().await {
            let daily_key = format!(
                "usage:daily:{}:{}",
                auth.user_id,
                chrono::Utc::now().format("%Y-%m-%d")
            );
            let val: Result<Option<i64>, _> =
                redis::AsyncCommands::get(&mut conn, &daily_key).await;
            if let Ok(Some(count)) = val {
                redis_today = count;
            }
        }
    }

    let today = redis_today.max(db_today);
    let plan = Plan::find_by_id(&state.db, &auth.plan)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let limit = plan.map(|p| p.requests_per_day).unwrap_or(100);
    Ok(Json(json!({
        "today": today, "this_week": week.max(today), "this_month": month.max(today),
        "daily_limit": limit,
        "remaining_today": (limit as i64 - today).max(0),
    })))
}

#[derive(Deserialize)]
pub struct HistoryQuery {
    pub days: Option<i32>,
}

/// GET /api/v1/usage/history
pub async fn history(
    State(state): State<AppState>,
    Query(q): Query<HistoryQuery>,
    request: axum::extract::Request,
) -> Result<Json<Value>, StatusCode> {
    let auth = request
        .extensions()
        .get::<AuthContext>()
        .cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let days = q.days.unwrap_or(30).min(90);
    let mut data = UsageLog::daily_history(&state.db, auth.user_id, days)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let today_str = chrono::Utc::now().format("%Y-%m-%d").to_string();
    if let Some(ref client) = state.redis {
        if let Ok(mut conn) = client.get_multiplexed_tokio_connection().await {
            let daily_key = format!("usage:daily:{}:{}", auth.user_id, today_str);
            let val: Result<Option<i64>, _> =
                redis::AsyncCommands::get(&mut conn, &daily_key).await;
            if let Ok(Some(redis_today)) = val {
                if let Some(entry) = data.iter_mut().find(|d| d.day == today_str) {
                    entry.count = entry.count.max(redis_today);
                } else if redis_today > 0 {
                    data.insert(
                        0,
                        crate::models::usage::DailyUsage {
                            day: today_str,
                            count: redis_today,
                        },
                    );
                }
            }
        }
    }

    Ok(Json(json!({ "history": data, "days": days })))
}
