use axum::{
    extract::{Path, State},
    Json,
};
use once_cell::sync::Lazy;
use serde_json::{json, Value};

use crate::api::state::AppState;
use crate::ingestion_subscriber::{self, CachedPrice};

static HTTP_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .unwrap_or_default()
});

type LatestPriceRow = (
    String,
    f64,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    String,
    String,
    Option<chrono::DateTime<chrono::Utc>>,
);

fn cached_price_from_row(row: LatestPriceRow) -> CachedPrice {
    let (symbol, price, bid, ask, volume, source, asset_type, received_at) = row;
    CachedPrice {
        symbol,
        price,
        bid,
        ask,
        volume,
        source,
        asset_type,
        received_at: received_at.map(|dt| dt.to_rfc3339()),
        updated_at: None,
    }
}

fn price_json(p: &CachedPrice) -> Value {
    json!({
        "symbol": p.symbol,
        "price": p.price,
        "bid": p.bid,
        "ask": p.ask,
        "volume": p.volume,
        "source": p.source,
        "asset_type": p.asset_type,
        "received_at": p.received_at,
    })
}

async fn load_latest_prices(pool: &sqlx::PgPool) -> Result<Vec<CachedPrice>, sqlx::Error> {
    let rows: Vec<LatestPriceRow> = sqlx::query_as(
        "SELECT symbol, price, bid, ask, volume, source, asset_type, received_at \
         FROM market.market_latest_prices \
         ORDER BY symbol",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(cached_price_from_row).collect())
}

async fn load_latest_price(
    pool: &sqlx::PgPool,
    symbol: &str,
) -> Result<Option<CachedPrice>, sqlx::Error> {
    let row: Option<LatestPriceRow> = sqlx::query_as(
        "SELECT symbol, price, bid, ask, volume, source, asset_type, received_at \
         FROM market.market_latest_prices \
         WHERE symbol = $1",
    )
    .bind(symbol)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(cached_price_from_row))
}

async fn latest_price_history_fallback(pool: &sqlx::PgPool, symbol: &str) -> Vec<Value> {
    match load_latest_price(pool, symbol).await {
        Ok(Some(p)) if p.price > 0.0 => {
            let now = chrono::Utc::now().timestamp();
            let start = now - (119 * 60);
            (0..120)
                .map(|i| {
                    json!({
                        "time": start + (i * 60),
                        "value": p.price,
                        "source": "last_known"
                    })
                })
                .collect()
        }
        Ok(_) => Vec::new(),
        Err(err) => {
            tracing::warn!(error = %err, symbol = %symbol, "failed to load latest price for history fallback");
            Vec::new()
        }
    }
}

fn tradingview_symbol(symbol: &str) -> String {
    let sym = symbol.trim().to_uppercase();
    match sym.as_str() {
        "XAUUSD" => "OANDA:XAUUSD".to_string(),
        "SPX" => "SP:SPX".to_string(),
        "DXY" => "TVC:DXY".to_string(),
        _ if sym.len() == 6 && sym.chars().all(|c| c.is_ascii_uppercase()) => format!("FX:{sym}"),
        _ => sym,
    }
}

fn encode_symbol(symbol: &str) -> String {
    symbol.replace(':', "%3A")
}

fn parse_reference_history(value: &Value) -> Vec<Value> {
    let mut history = parse_bar_array(value)
        .or_else(|| value.get("bars").and_then(parse_bar_array))
        .or_else(|| parse_compact_history(value))
        .or_else(|| parse_yahoo_history(value))
        .unwrap_or_default();

    let limit = 120;
    if history.len() > limit {
        history = history.split_off(history.len() - limit);
    }
    history
}

fn parse_bar_array(value: &Value) -> Option<Vec<Value>> {
    let bars = value.as_array()?;
    let history: Vec<Value> = bars
        .iter()
        .filter_map(|bar| {
            let time = bar
                .get("time")
                .or_else(|| bar.get("timestamp"))
                .and_then(as_timestamp)?;
            let close = bar
                .get("close")
                .or_else(|| bar.get("value"))
                .and_then(as_price)?;
            Some(json!({ "time": time, "value": close }))
        })
        .collect();

    if history.is_empty() {
        None
    } else {
        Some(history)
    }
}

fn parse_compact_history(value: &Value) -> Option<Vec<Value>> {
    let timestamps = value.get("t")?.as_array()?;
    let closes = value.get("c")?.as_array()?;
    let len = timestamps.len().min(closes.len());
    let history: Vec<Value> = (0..len)
        .filter_map(|i| {
            Some(json!({
                "time": as_timestamp(&timestamps[i])?,
                "value": as_price(&closes[i])?,
            }))
        })
        .collect();

    if history.is_empty() {
        None
    } else {
        Some(history)
    }
}

fn parse_yahoo_history(value: &Value) -> Option<Vec<Value>> {
    let result = value.get("chart")?.get("result")?.get(0)?;
    let timestamps = result.get("timestamp")?.as_array()?;
    let closes = result
        .get("indicators")?
        .get("quote")?
        .get(0)?
        .get("close")?
        .as_array()?;
    let len = timestamps.len().min(closes.len());
    let history: Vec<Value> = (0..len)
        .filter_map(|i| {
            Some(json!({
                "time": as_timestamp(&timestamps[i])?,
                "value": as_price(&closes[i])?,
            }))
        })
        .collect();

    if history.is_empty() {
        None
    } else {
        Some(history)
    }
}

fn as_timestamp(value: &Value) -> Option<i64> {
    match value {
        Value::Number(n) => n
            .as_i64()
            .map(|ts| if ts > 10_000_000_000 { ts / 1000 } else { ts }),
        Value::String(s) => {
            s.parse::<i64>()
                .ok()
                .map(|ts| if ts > 10_000_000_000 { ts / 1000 } else { ts })
        }
        _ => None,
    }
}

fn as_price(value: &Value) -> Option<f64> {
    match value {
        Value::Number(n) => n.as_f64().filter(|price| *price > 0.0),
        Value::String(s) => s.parse::<f64>().ok().filter(|price| *price > 0.0),
        _ => None,
    }
}

fn history_is_usable(history: &[Value], now: chrono::DateTime<chrono::Utc>) -> bool {
    if history.len() < 5 {
        return false;
    }

    let last_time = history
        .last()
        .and_then(|point| point.get("time"))
        .and_then(as_timestamp);
    if last_time.is_some_and(|ts| now.timestamp() - ts > 6 * 60 * 60) {
        return false;
    }

    let prices: Vec<f64> = history
        .iter()
        .filter_map(|point| point.get("value").and_then(as_price))
        .collect();
    if prices.len() < 5 {
        return false;
    }

    let min = prices.iter().copied().fold(f64::INFINITY, f64::min);
    let max = prices.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let latest = prices.last().copied().unwrap_or(0.0).abs().max(1.0);
    ((max - min) / latest) >= 0.00001
}

pub async fn list_prices(State(state): State<AppState>) -> Json<Value> {
    let mut prices = ingestion_subscriber::get_all_prices();

    if prices.is_empty() {
        match load_latest_prices(&state.db).await {
            Ok(db_prices) => {
                for price in &db_prices {
                    ingestion_subscriber::set_price(price.clone());
                }
                prices = db_prices;
            }
            Err(err) => {
                tracing::warn!(error = %err, "failed to load latest market prices from database");
            }
        }
    }

    let items: Vec<Value> = prices.iter().map(price_json).collect();

    Json(json!({
        "items": items,
        "total": items.len(),
    }))
}

pub async fn get_price(State(state): State<AppState>, Path(symbol): Path<String>) -> Json<Value> {
    if let Some(p) = ingestion_subscriber::get_price(&symbol) {
        return Json(price_json(&p));
    }

    let sym = symbol.to_uppercase();
    match load_latest_price(&state.db, &sym).await {
        Ok(Some(p)) => {
            ingestion_subscriber::set_price(p.clone());
            Json(price_json(&p))
        }
        Ok(None) => Json(json!({
            "error": format!("Symbol '{}' not found in price cache or latest price store. Available symbols can be retrieved from GET /api/v1/market/prices", sym),
        })),
        Err(err) => {
            tracing::warn!(error = %err, symbol = %sym, "failed to load latest market price from database");
            Json(json!({
                "error": format!("Symbol '{}' not found in price cache. Available symbols can be retrieved from GET /api/v1/market/prices", sym),
            }))
        }
    }
}

pub async fn get_history(State(state): State<AppState>, Path(symbol): Path<String>) -> Json<Value> {
    let sym = symbol.to_uppercase();

    if let Some(clickhouse) = &state.clickhouse {
        match clickhouse.latest_history(&sym, 120).await {
            Ok(history) if history_is_usable(&history, chrono::Utc::now()) => {
                return Json(json!(history));
            }
            Ok(history) if !history.is_empty() => {
                tracing::warn!(symbol = %sym, points = history.len(), "ClickHouse market history is flat or stale, falling back");
            }
            Ok(_) => {}
            Err(err) => {
                tracing::warn!(error = %err, symbol = %sym, "failed to query ClickHouse market history");
            }
        }
    }

    let db_res: Result<Vec<(chrono::DateTime<chrono::Utc>, f64)>, _> = sqlx::query_as(
        "SELECT time, close FROM market.ohlcv_candles \
         WHERE symbol = $1 AND resolution = '1m' \
         ORDER BY time DESC LIMIT 120",
    )
    .bind(&sym)
    .fetch_all(&state.db)
    .await;

    if let Ok(candles) = db_res {
        if !candles.is_empty() {
            let mut history: Vec<Value> = candles
                .into_iter()
                .map(|(t, close)| {
                    json!({
                        "time": t.timestamp(),
                        "value": close
                    })
                })
                .collect();
            history.reverse();
            if history_is_usable(&history, chrono::Utc::now()) {
                return Json(json!(history));
            }
            tracing::warn!(symbol = %sym, points = history.len(), "market history DB candles are flat or stale, falling back to reference history");
        }
    }

    if sym.ends_with("USDT") {
        let Ok(template) = std::env::var("CRYPTO_HISTORY_URL_TEMPLATE") else {
            return Json(json!(latest_price_history_fallback(&state.db, &sym).await));
        };
        let url = template.replace("{symbol}", &sym);
        match HTTP_CLIENT.get(&url).send().await {
            Ok(res) => {
                let status = res.status();
                if !status.is_success() {
                    tracing::warn!("crypto history HTTP error for {}: {}", sym, status);
                    return Json(json!(latest_price_history_fallback(&state.db, &sym).await));
                }
                match res.json::<Vec<Vec<Value>>>().await {
                    Ok(data) => {
                        let history: Vec<Value> = data
                            .iter()
                            .filter_map(|item| {
                                if item.len() >= 5 {
                                    let time_ms = item[0].as_i64()?;
                                    let close_str = item[4].as_str()?;
                                    let close_val: f64 = close_str.parse().ok()?;
                                    Some(json!({
                                        "time": time_ms / 1000,
                                        "value": close_val
                                    }))
                                } else {
                                    None
                                }
                            })
                            .collect();
                        return Json(json!(history));
                    }
                    Err(err) => {
                        tracing::warn!(
                            "Failed to parse crypto history JSON for {}: {:?}",
                            sym,
                            err
                        );
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to fetch crypto history for {}: {:?}", sym, e);
            }
        }
    }

    let reference_symbol = tradingview_symbol(&sym);
    let template = std::env::var("TRADINGVIEW_HISTORY_URL_TEMPLATE")
        .or_else(|_| std::env::var("REFERENCE_HISTORY_URL_TEMPLATE"))
        .unwrap_or_default();
    if template.trim().is_empty() {
        tracing::warn!(symbol = %sym, ticker = %reference_symbol, "reference history URL template not configured, using last known price fallback");
        return Json(json!(latest_price_history_fallback(&state.db, &sym).await));
    }
    let url = template
        .trim()
        .replace("{symbol}", &encode_symbol(&reference_symbol));

    match HTTP_CLIENT.get(&url).send().await {
        Ok(res) => {
            let status = res.status();
            if !status.is_success() {
                tracing::warn!(
                    "reference history HTTP error for {} (ticker: {}): {}",
                    sym,
                    reference_symbol,
                    status
                );
                return Json(json!(latest_price_history_fallback(&state.db, &sym).await));
            }
            match res.json::<Value>().await {
                Ok(json_data) => {
                    let history = parse_reference_history(&json_data);
                    if !history.is_empty() {
                        return Json(json!(history));
                    }
                    tracing::warn!(
                        "reference history JSON structure mismatch for {} (ticker: {})",
                        sym,
                        reference_symbol
                    );
                }
                Err(err) => {
                    tracing::warn!(
                        "Failed to parse reference history JSON for {} (ticker: {}): {:?}",
                        sym,
                        reference_symbol,
                        err
                    );
                }
            }
        }
        Err(e) => {
            tracing::warn!(
                "Failed to fetch reference history for {} (ticker: {}): {:?}",
                sym,
                reference_symbol,
                e
            );
        }
    }

    Json(json!(latest_price_history_fallback(&state.db, &sym).await))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn maps_symbols_to_tradingview() {
        assert_eq!(tradingview_symbol("XAUUSD"), "OANDA:XAUUSD");
        assert_eq!(tradingview_symbol("SPX"), "SP:SPX");
        assert_eq!(tradingview_symbol("DXY"), "TVC:DXY");
        assert_eq!(tradingview_symbol("EURUSD"), "FX:EURUSD");
    }

    #[test]
    fn parses_history_shapes() {
        let bars = parse_reference_history(&json!([
            { "time": 1_700_000_000, "close": 2000.5 },
            { "timestamp": 1_700_000_060_000i64, "value": "2001.5" }
        ]));
        assert_eq!(bars.len(), 2);
        assert_eq!(bars[1]["time"], 1_700_000_060);
        assert_eq!(bars[1]["value"], 2001.5);

        let compact = parse_reference_history(&json!({
            "t": [1_700_000_000, 1_700_000_060],
            "c": [1.08, 1.09]
        }));
        assert_eq!(compact.len(), 2);
        assert_eq!(compact[0]["value"], 1.08);
    }

    #[test]
    fn rejects_flat_or_stale_history() {
        let now = chrono::Utc::now();
        let start = now.timestamp() - 4 * 60;
        let flat: Vec<Value> = (0..5)
            .map(|i| json!({ "time": start + i * 60, "value": 100.0 }))
            .collect();
        let moving: Vec<Value> = (0..5)
            .map(|i| json!({ "time": start + i * 60, "value": 100.0 + i as f64 * 0.1 }))
            .collect();
        let stale: Vec<Value> = (0..5)
            .map(|i| json!({ "time": start - 7 * 60 * 60 + i * 60, "value": 100.0 + i as f64 * 0.1 }))
            .collect();

        assert!(!history_is_usable(&flat, now));
        assert!(history_is_usable(&moving, now));
        assert!(!history_is_usable(&stale, now));
    }
}
