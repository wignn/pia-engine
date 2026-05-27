use axum::{extract::State, Json};
use serde_json::json;

use crate::state::AppState;

pub async fn data_quality(State(state): State<AppState>) -> Json<serde_json::Value> {
    let prices = state.prices.read().len();
    let items = if let Some(clickhouse) = &state.clickhouse {
        match clickhouse.tick_stats().await {
            Ok(rows) => rows
                .into_iter()
                .map(|row| {
                    json!({
                        "symbol": row.symbol,
                        "ticks_5m": row.ticks_5m,
                        "latest_at": row.latest_at,
                        "status": if row.ticks_5m > 0 { "ok" } else { "stale" },
                    })
                })
                .collect::<Vec<_>>(),
            Err(err) => {
                tracing::warn!(error = %err, "failed to load tick quality stats");
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };

    Json(json!({
        "items": items,
        "total": items.len(),
        "cached_prices": prices,
        "generated_at": chrono::Utc::now().to_rfc3339(),
    }))
}
