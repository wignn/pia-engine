use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::prices::{load_latest_price, CachedPrice};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub resolution: Option<String>,
    pub limit: Option<usize>,
}

pub async fn get_history(
    Path(symbol): Path<String>,
    Query(query): Query<HistoryQuery>,
    State(state): State<AppState>,
) -> Json<Value> {
    let symbol = symbol.to_uppercase();
    let resolution = normalize_resolution(query.resolution.as_deref().unwrap_or("1m"));
    let limit = query.limit.unwrap_or(120).clamp(1, 1000);

    if let Some(clickhouse) = &state.clickhouse {
        match clickhouse.latest_history(&symbol, &resolution, limit).await {
            Ok(history) if !history.is_empty() => {
                return Json(json!(history));
            }
            Ok(_) => {}
            Err(err) => {
                tracing::warn!(error = %err, symbol = %symbol, "failed to load ClickHouse history")
            }
        }
    }

    let history = match postgres_history(&state.db, &symbol, &resolution, limit).await {
        Ok(history) if !history.is_empty() => history,
        Ok(_) => latest_price_history_fallback(&state, &symbol, &resolution).await,
        Err(err) => {
            tracing::warn!(error = %err, symbol = %symbol, "failed to load Postgres history");
            latest_price_history_fallback(&state, &symbol, &resolution).await
        }
    };
    Json(json!(history))
}

pub fn normalize_resolution(raw: &str) -> String {
    match raw.trim().to_lowercase().as_str() {
        "1" | "1m" | "m1" => "1m".to_string(),
        "5" | "5m" | "m5" => "5m".to_string(),
        "15" | "15m" | "m15" => "15m".to_string(),
        "60" | "1h" | "h1" => "1h".to_string(),
        _ => "1m".to_string(),
    }
}

fn resolution_bucket_seconds(resolution: &str) -> i64 {
    match resolution {
        "5m" => 5 * 60,
        "15m" => 15 * 60,
        "1h" => 60 * 60,
        _ => 60,
    }
}

async fn postgres_history(
    pool: &sqlx::PgPool,
    symbol: &str,
    resolution: &str,
    limit: usize,
) -> Result<Vec<Value>, sqlx::Error> {
    let rows: Vec<(chrono::DateTime<chrono::Utc>, f64)> = if resolution == "1m" {
        sqlx::query_as(
            "SELECT time, close FROM market.ohlcv_candles WHERE symbol = $1 AND resolution = '1m' ORDER BY time DESC LIMIT $2",
        )
        .bind(symbol)
        .bind(limit as i64)
        .fetch_all(pool)
        .await?
    } else {
        let bucket_seconds = resolution_bucket_seconds(resolution);
        sqlx::query_as(
            "WITH bucketed AS (
                SELECT to_timestamp(floor(extract(epoch from time) / $2) * $2) AT TIME ZONE 'UTC' AS bucket_time, time, close
                FROM market.ohlcv_candles
                WHERE symbol = $1 AND resolution = '1m'
            ), ranked AS (
                SELECT bucket_time, close, row_number() OVER (PARTITION BY bucket_time ORDER BY time DESC) AS rn
                FROM bucketed
            )
            SELECT bucket_time, close FROM ranked WHERE rn = 1 ORDER BY bucket_time DESC LIMIT $3",
        )
        .bind(symbol)
        .bind(bucket_seconds as f64)
        .bind(limit as i64)
        .fetch_all(pool)
        .await?
    };

    Ok(rows
        .into_iter()
        .rev()
        .map(|(time, value)| json!({ "time": time.timestamp(), "value": value }))
        .collect())
}

async fn latest_price_history_fallback(
    state: &AppState,
    symbol: &str,
    resolution: &str,
) -> Vec<Value> {
    if resolution != "1m" {
        return Vec::new();
    }

    match load_latest_price(&state.db, symbol).await {
        Ok(Some(price)) if should_emit_last_known_fallback(&price, &state.calendar) => {
            let now = chrono::Utc::now().timestamp();
            let start = now - (119 * 60);
            (0..120)
                .map(|i| json!({ "time": start + (i * 60), "value": price.price, "source": "last_known" }))
                .collect()
        }
        Ok(_) => Vec::new(),
        Err(err) => {
            tracing::warn!(error = %err, symbol = %symbol, "failed to load latest price for history fallback");
            Vec::new()
        }
    }
}

fn should_emit_last_known_fallback(
    price: &CachedPrice,
    calendar: &crate::calendar::CalendarCache,
) -> bool {
    if price.price <= 0.0 {
        return false;
    }

    crate::session::session_status(
        &price.symbol,
        &price.asset_type,
        chrono::Utc::now(),
        Some(calendar),
    )
    .is_open
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_supported_resolutions() {
        assert_eq!(normalize_resolution("M1"), "1m");
        assert_eq!(normalize_resolution("5"), "5m");
        assert_eq!(normalize_resolution("h1"), "1h");
        assert_eq!(normalize_resolution("bad"), "1m");
        assert_eq!(resolution_bucket_seconds("5m"), 300);
        assert_eq!(resolution_bucket_seconds("15m"), 900);
        assert_eq!(resolution_bucket_seconds("1h"), 3600);
    }

    #[test]
    fn suppresses_last_known_fallback_for_closed_sessions() {
        let calendar = crate::calendar::CalendarCache::default();
        let price = CachedPrice {
            symbol: "SPX".to_string(),
            price: 7519.11,
            bid: None,
            ask: None,
            volume: None,
            source: "market_data".to_string(),
            asset_type: "index".to_string(),
            received_at: None,
        };

        assert!(!should_emit_last_known_fallback(&price, &calendar));
    }

    #[test]
    fn allows_last_known_fallback_for_crypto_sessions() {
        let calendar = crate::calendar::CalendarCache::default();
        let price = CachedPrice {
            symbol: "BTCUSDT".to_string(),
            price: 100_000.0,
            bid: None,
            ask: None,
            volume: None,
            source: "market_data".to_string(),
            asset_type: "crypto".to_string(),
            received_at: None,
        };

        assert!(should_emit_last_known_fallback(&price, &calendar));
    }
}
