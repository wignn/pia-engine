use std::collections::HashMap;

use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{spikes, state::AppState};

#[derive(Debug, Deserialize)]
pub struct AlertQuery {
    pub window: Option<String>,
    pub threshold: Option<f64>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Alert {
    pub kind: &'static str,
    pub symbol: String,
    pub asset_type: String,
    pub severity: &'static str,
    pub direction: &'static str,
    pub latest_price: f64,
    pub baseline_price: f64,
    pub move_pct: f64,
    pub threshold_pct: f64,
    pub tick_count: u64,
    pub ticks_5m: u64,
    pub latest_at: String,
    pub reason: String,
    pub recommended_action: &'static str,
}

pub async fn alerts(Query(query): Query<AlertQuery>, State(state): State<AppState>) -> Json<Value> {
    let window = query.window.as_deref().unwrap_or("5m");
    let window_minutes = spikes::spike_window_minutes(window);
    let threshold = query
        .threshold
        .unwrap_or_else(|| spikes::spike_threshold(window_minutes));
    let limit = query.limit.unwrap_or(25).clamp(1, 100);
    let alerts = collect_alerts(&state, window_minutes, threshold, limit).await;

    Json(json!({
        "items": alerts,
        "total": alerts.len(),
        "window": format!("{window_minutes}m"),
        "threshold_pct": threshold,
        "generated_at": chrono::Utc::now().to_rfc3339(),
    }))
}

pub async fn collect_alerts(
    state: &AppState,
    window_minutes: u32,
    threshold: f64,
    limit: usize,
) -> Vec<Alert> {
    let tick_stats = if let Some(clickhouse) = &state.clickhouse {
        clickhouse.tick_stats().await.unwrap_or_default()
    } else {
        Vec::new()
    };
    let tick_stats_by_symbol = tick_stats
        .into_iter()
        .map(|row| (row.symbol.clone(), row))
        .collect::<HashMap<_, _>>();

    let Some(clickhouse) = &state.clickhouse else {
        return Vec::new();
    };

    match clickhouse
        .spike_candidates(window_minutes, threshold, limit)
        .await
    {
        Ok(rows) => rows
            .into_iter()
            .map(|row| {
                let ticks_5m = tick_stats_by_symbol
                    .get(&row.symbol)
                    .map(|stats| stats.ticks_5m)
                    .unwrap_or(0);
                let severity = alert_severity(row.move_pct, ticks_5m);
                Alert {
                    kind: "volatility_spike",
                    symbol: row.symbol,
                    asset_type: row.asset_type,
                    severity,
                    direction: if row.move_pct >= 0.0 { "up" } else { "down" },
                    latest_price: row.latest_price,
                    baseline_price: row.baseline_price,
                    move_pct: row.move_pct,
                    threshold_pct: threshold,
                    tick_count: row.tick_count,
                    ticks_5m,
                    latest_at: row.latest_at,
                    reason: alert_reason(row.move_pct, ticks_5m),
                    recommended_action: recommended_action(severity),
                }
            })
            .collect(),
        Err(err) => {
            tracing::warn!(error = %err, "failed to load alerts");
            Vec::new()
        }
    }
}

pub fn is_notifiable_severity(severity: &str) -> bool {
    matches!(severity, "critical" | "high")
}

fn alert_severity(move_pct: f64, ticks_5m: u64) -> &'static str {
    let abs = move_pct.abs();
    if abs >= 1.0 && ticks_5m >= 10 {
        "critical"
    } else if abs >= 0.5 || (abs >= 0.2 && ticks_5m >= 20) {
        "high"
    } else if abs >= 0.2 || ticks_5m >= 10 {
        "medium"
    } else {
        "low"
    }
}

fn alert_reason(move_pct: f64, ticks_5m: u64) -> String {
    let direction = if move_pct >= 0.0 { "up" } else { "down" };
    format!(
        "Price moved {direction} by {:.2}% with {ticks_5m} ticks in the last 5 minutes.",
        move_pct.abs()
    )
}

fn recommended_action(severity: &str) -> &'static str {
    match severity {
        "critical" => "Escalate immediately and check related news/calendar before acting.",
        "high" => "Monitor closely and confirm with news or cross-asset movement.",
        "medium" => "Watch for continuation; avoid alert spam until confirmation.",
        _ => "Informational only.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alert_severity_combines_move_and_tick_activity() {
        assert_eq!(alert_severity(1.2, 20), "critical");
        assert_eq!(alert_severity(0.25, 25), "high");
        assert_eq!(alert_severity(0.25, 1), "medium");
        assert_eq!(alert_severity(0.01, 1), "low");
    }

    #[test]
    fn only_high_and_critical_alerts_are_notifiable() {
        assert!(is_notifiable_severity("critical"));
        assert!(is_notifiable_severity("high"));
        assert!(!is_notifiable_severity("medium"));
    }
}
