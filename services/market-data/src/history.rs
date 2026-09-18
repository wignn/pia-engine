use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::cache::HistoryCacheKey;
use crate::prices::CachedPrice;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub resolution: Option<String>,
    pub limit: Option<usize>,
    pub before: Option<i64>,
}

pub async fn get_history(
    Path(symbol): Path<String>,
    Query(query): Query<HistoryQuery>,
    State(state): State<AppState>,
) -> (StatusCode, Json<Value>) {
    let symbol = symbol.to_uppercase();
    let resolution = normalize_resolution(query.resolution.as_deref().unwrap_or("1m"));
    let limit = query.limit.unwrap_or(120).clamp(1, 1000);

    let Some(clickhouse) = &state.clickhouse else {
        tracing::error!(symbol = %symbol, "ClickHouse is required for market history");
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": "market_history_unavailable", "retryable": true})),
        );
    };

    let key = HistoryCacheKey {
        symbol: symbol.clone(),
        resolution: resolution.clone(),
        limit,
        before: query.before,
    };

    let clickhouse = clickhouse.clone();
    let symbol_fetch = symbol.clone();
    let res_fetch = resolution.clone();
    let before_fetch = query.before;

    let history_res = state
        .history_cache
        .get_or_fetch(key, move || async move {
            clickhouse
                .latest_history(&symbol_fetch, &res_fetch, limit, before_fetch)
                .await
        })
        .await;

    match history_res {
        Ok(cached_page) => {
            if query.before.is_none() {
                let live_opt = state.prices.read().get(&symbol).cloned();
                if let Some(live) = live_opt {
                    let mut page = (*cached_page).clone();
                    stitch_active_candle(&mut page.items, &live, &resolution);
                    return (StatusCode::OK, Json(json!(page)));
                }
            }
            (StatusCode::OK, Json(json!(*cached_page)))
        }
        Err(err) => {
            tracing::warn!(error = %err, symbol = %symbol, "failed to load ClickHouse history");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({"error": "market_history_unavailable", "retryable": true})),
            )
        }
    }
}

fn resolution_to_seconds(res: &str) -> i64 {
    match res.trim() {
        "1m" | "1" => 60,
        "5m" | "5" => 300,
        "15m" | "15" => 900,
        "30m" | "30" => 1800,
        "1h" | "60" => 3600,
        "4h" | "240" => 14400,
        "1d" | "D" | "1D" => 86400,
        "1w" | "W" | "1W" => 604800,
        _ => 60,
    }
}

pub fn stitch_active_candle(items: &mut Vec<Value>, live: &CachedPrice, resolution: &str) {
    let now_ts = live
        .timestamp_ms
        .map(|ms| ms / 1000)
        .unwrap_or_else(|| chrono::Utc::now().timestamp());
    let interval_secs = resolution_to_seconds(resolution);
    let current_bucket = (now_ts / interval_secs) * interval_secs;

    if let Some(latest) = items.last_mut() {
        if let Some(candle_time) = latest.get("time").and_then(Value::as_i64) {
            if candle_time == current_bucket {
                if let Some(obj) = latest.as_object_mut() {
                    let high = obj
                        .get("high")
                        .and_then(Value::as_f64)
                        .unwrap_or(live.price);
                    let low = obj.get("low").and_then(Value::as_f64).unwrap_or(live.price);
                    obj.insert("close".to_string(), json!(live.price));
                    obj.insert("value".to_string(), json!(live.price));
                    obj.insert("high".to_string(), json!(high.max(live.price)));
                    obj.insert("low".to_string(), json!(low.min(live.price)));
                    if let Some(v) = live.volume {
                        let existing_vol = obj.get("volume").and_then(Value::as_f64).unwrap_or(0.0);
                        obj.insert("volume".to_string(), json!(existing_vol + v));
                    }
                    if let Some(rec) = &live.received_at {
                        obj.insert("latest_at".to_string(), json!(rec));
                    }
                }
            } else if current_bucket > candle_time {
                let new_candle = json!({
                    "time": current_bucket,
                    "open": live.price,
                    "high": live.price,
                    "low": live.price,
                    "close": live.price,
                    "value": live.price,
                    "volume": live.volume.unwrap_or(0.0),
                    "tick_count": 1,
                    "latest_at": live.received_at.clone().unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
                    "source": "live_buffer"
                });
                items.push(new_candle);
            }
        }
    } else {
        // Items was completely empty: create the active candle as first candle
        let new_candle = json!({
            "time": current_bucket,
            "open": live.price,
            "high": live.price,
            "low": live.price,
            "close": live.price,
            "value": live.price,
            "volume": live.volume.unwrap_or(0.0),
            "tick_count": 1,
            "latest_at": live.received_at.clone().unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
            "source": "live_buffer"
        });
        items.push(new_candle);
    }
}

pub fn normalize_resolution(raw: &str) -> String {
    match raw.trim().to_lowercase().as_str() {
        "1" | "1m" | "m1" => "1m".to_string(),
        "5" | "5m" | "m5" => "5m".to_string(),
        "15" | "15m" | "m15" => "15m".to_string(),
        "30" | "30m" | "m30" => "30m".to_string(),
        "60" | "1h" | "h1" => "1h".to_string(),
        "240" | "4h" | "h4" => "4h".to_string(),
        "d" | "1d" => "1D".to_string(),
        "w" | "1w" => "1W".to_string(),
        _ => "1m".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_supported_resolutions() {
        assert_eq!(normalize_resolution("M1"), "1m");
        assert_eq!(normalize_resolution("5"), "5m");
        assert_eq!(normalize_resolution("15"), "15m");
        assert_eq!(normalize_resolution("30"), "30m");
        assert_eq!(normalize_resolution("30m"), "30m");
        assert_eq!(normalize_resolution("M30"), "30m");
        assert_eq!(normalize_resolution("h1"), "1h");
        assert_eq!(normalize_resolution("bad"), "1m");
    }

    #[test]
    fn test_stitch_active_candle_updates_existing_bucket() {
        let mut items = vec![json!({
            "time": 1700000040,
            "open": 50000.0,
            "high": 50100.0,
            "low": 49900.0,
            "close": 50050.0,
            "value": 50050.0,
            "volume": 10.0,
            "source": "clickhouse_candles_1m"
        })];

        let live = CachedPrice {
            symbol: "BTCUSDT".to_string(),
            price: 50200.0,
            bid: None,
            ask: None,
            volume: Some(2.5),
            source: "binance".to_string(),
            asset_type: "crypto".to_string(),
            received_at: Some("2023-11-14T22:14:00Z".to_string()),
            timestamp_ms: Some((1700000040 + 20) * 1000), // within minute 1700000040
            feed: None,
        };

        stitch_active_candle(&mut items, &live, "1m");

        let latest = items.last().unwrap();
        assert_eq!(latest["close"], 50200.0);
        assert_eq!(latest["high"], 50200.0);
        assert_eq!(latest["volume"], 12.5);
    }

    #[test]
    fn test_stitch_active_candle_appends_new_bucket() {
        let mut items = vec![json!({
            "time": 1700000040,
            "open": 50000.0,
            "high": 50100.0,
            "low": 49900.0,
            "close": 50050.0,
            "value": 50050.0,
            "volume": 10.0,
            "source": "clickhouse_candles_1m"
        })];

        let live = CachedPrice {
            symbol: "BTCUSDT".to_string(),
            price: 50300.0,
            bid: None,
            ask: None,
            volume: Some(1.0),
            source: "binance".to_string(),
            asset_type: "crypto".to_string(),
            received_at: Some("2023-11-14T22:15:10Z".to_string()),
            timestamp_ms: Some((1700000040 + 60 + 10) * 1000), // next minute 1700000100
            feed: None,
        };

        stitch_active_candle(&mut items, &live, "1m");

        assert_eq!(items.len(), 2);
        let latest = items.last().unwrap();
        assert_eq!(latest["time"], 1700000100);
        assert_eq!(latest["close"], 50300.0);
        assert_eq!(latest["source"], "live_buffer");
    }

    #[test]
    fn test_stitch_active_candle_30m_resolution() {
        // 30m = 1800s
        let base_30m_bucket = 1700001000;
        let mut items = vec![json!({
            "time": base_30m_bucket,
            "open": 50000.0,
            "high": 50100.0,
            "low": 49900.0,
            "close": 50050.0,
            "value": 50050.0,
            "volume": 100.0,
            "source": "clickhouse_candles_rollup"
        })];

        // 1. Tick within the same 30m window updates current candle
        let live_same_bucket = CachedPrice {
            symbol: "BTCUSDT".to_string(),
            price: 50400.0,
            bid: None,
            ask: None,
            volume: Some(5.0),
            source: "binance".to_string(),
            asset_type: "crypto".to_string(),
            received_at: Some("2023-11-14T22:10:00Z".to_string()),
            timestamp_ms: Some((base_30m_bucket + 300) * 1000),
            feed: None,
        };

        stitch_active_candle(&mut items, &live_same_bucket, "30m");
        assert_eq!(items.len(), 1);
        let latest = items.last().unwrap();
        assert_eq!(latest["time"], base_30m_bucket);
        assert_eq!(latest["close"], 50400.0);
        assert_eq!(latest["high"], 50400.0);
        assert_eq!(latest["volume"], 105.0);

        // 2. Tick in the next 30m window appends a new candle
        let live_next_bucket = CachedPrice {
            symbol: "BTCUSDT".to_string(),
            price: 50600.0,
            bid: None,
            ask: None,
            volume: Some(12.0),
            source: "binance".to_string(),
            asset_type: "crypto".to_string(),
            received_at: Some("2023-11-14T22:31:00Z".to_string()),
            timestamp_ms: Some((base_30m_bucket + 1800 + 60) * 1000),
            feed: None,
        };

        stitch_active_candle(&mut items, &live_next_bucket, "30m");
        assert_eq!(items.len(), 2);
        let latest = items.last().unwrap();
        assert_eq!(latest["time"], base_30m_bucket + 1800);
        assert_eq!(latest["open"], 50600.0);
        assert_eq!(latest["close"], 50600.0);
        assert_eq!(latest["high"], 50600.0);
        assert_eq!(latest["low"], 50600.0);
        assert_eq!(latest["volume"], 12.0);
        assert_eq!(latest["source"], "live_buffer");
    }
}
