use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct SpikeQuery {
    pub window: Option<String>,
    pub threshold: Option<f64>,
    pub limit: Option<usize>,
}

pub async fn spikes(Query(query): Query<SpikeQuery>, State(state): State<AppState>) -> Json<Value> {
    let window_minutes = spike_window_minutes(query.window.as_deref().unwrap_or("5m"));
    let threshold = query
        .threshold
        .unwrap_or_else(|| spike_threshold(window_minutes));
    let limit = query.limit.unwrap_or(25);

    let rows = if let Some(clickhouse) = &state.clickhouse {
        match clickhouse
            .spike_candidates(window_minutes, threshold, limit)
            .await
        {
            Ok(rows) => rows
                .into_iter()
                .map(|row| {
                    json!({
                        "symbol": row.symbol,
                        "asset_type": row.asset_type,
                        "latest_price": row.latest_price,
                        "baseline_price": row.baseline_price,
                        "move_pct": row.move_pct,
                        "tick_count": row.tick_count,
                        "latest_at": row.latest_at,
                        "severity": spike_severity(row.move_pct),
                    })
                })
                .collect::<Vec<_>>(),
            Err(err) => {
                tracing::warn!(error = %err, "failed to load volatility spikes");
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };

    Json(json!({
        "items": rows,
        "total": rows.len(),
        "window": format!("{window_minutes}m"),
        "generated_at": chrono::Utc::now().to_rfc3339(),
    }))
}

pub fn spike_window_minutes(window: &str) -> u32 {
    match window.trim().to_lowercase().as_str() {
        "1m" | "1" => 1,
        "5m" | "5" => 5,
        "15m" | "15" => 15,
        "30m" | "30" => 30,
        "1h" | "60" => 60,
        _ => 5,
    }
}

pub fn spike_threshold(window_minutes: u32) -> f64 {
    match window_minutes {
        0..=1 => 0.05,
        2..=5 => 0.10,
        6..=15 => 0.20,
        _ => 0.35,
    }
}

pub fn spike_severity(move_pct: f64) -> &'static str {
    let abs = move_pct.abs();
    if abs >= 1.0 {
        "critical"
    } else if abs >= 0.5 {
        "high"
    } else if abs >= 0.2 {
        "medium"
    } else {
        "low"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_spike_windows_and_thresholds() {
        assert_eq!(spike_window_minutes("1h"), 60);
        assert_eq!(spike_window_minutes("bad"), 5);
        assert_eq!(spike_threshold(5), 0.10);
        assert_eq!(spike_severity(1.2), "critical");
    }
}
