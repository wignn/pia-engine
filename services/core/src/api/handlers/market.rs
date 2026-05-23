use axum::{
    extract::{Path, State},
    Json,
};
use once_cell::sync::Lazy;
use serde_json::{json, Value};

use crate::api::state::AppState;
use crate::ingestion_subscriber;

static HTTP_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .unwrap_or_default()
});

pub async fn list_prices(_state: State<AppState>) -> Json<Value> {
    let prices = ingestion_subscriber::get_all_prices();

    let items: Vec<Value> = prices
        .iter()
        .map(|p| {
            json!({
                "symbol": p.symbol,
                "price": p.price,
                "bid": p.bid,
                "ask": p.ask,
                "volume": p.volume,
                "source": "market_data",
                "asset_type": p.asset_type,
                "received_at": p.received_at,
            })
        })
        .collect();

    Json(json!({
        "items": items,
        "total": items.len(),
    }))
}

/// GET /api/v1/market/prices/{symbol}
/// Returns the cached price for a single symbol (case-insensitive).
/// Example: /api/v1/market/prices/XAUUSD
pub async fn get_price(_state: State<AppState>, Path(symbol): Path<String>) -> Json<Value> {
    match ingestion_subscriber::get_price(&symbol) {
        Some(p) => Json(json!({
            "symbol": p.symbol,
            "price": p.price,
            "bid": p.bid,
            "ask": p.ask,
            "volume": p.volume,
            "source": p.source,
            "asset_type": p.asset_type,
            "received_at": p.received_at,
        })),
        None => Json(json!({
            "error": format!("Symbol '{}' not found in price cache. Available symbols can be retrieved from GET /api/v1/market/prices", symbol.to_uppercase()),
        })),
    }
}

pub async fn get_history(State(state): State<AppState>, Path(symbol): Path<String>) -> Json<Value> {
    let sym = symbol.to_uppercase();

    // Try reading from our separate local time-series database first
    let db_res: Result<Vec<(chrono::DateTime<chrono::Utc>, f64)>, _> = sqlx::query_as(
        "SELECT time, close FROM ohlcv_candles \
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
            return Json(json!(history));
        }
    }

    if sym.ends_with("USDT") {
        let Ok(template) = std::env::var("CRYPTO_HISTORY_URL_TEMPLATE") else {
            return Json(json!([]));
        };
        let url = template.replace("{symbol}", &sym);
        match HTTP_CLIENT.get(&url).send().await {
            Ok(res) => {
                let status = res.status();
                if !status.is_success() {
                    tracing::warn!("crypto history HTTP error for {}: {}", sym, status);
                    return Json(json!([]));
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

    let reference_symbol = match sym.as_str() {
        "XAUUSD" => "GC=F".to_string(),
        "SPX" => "^GSPC".to_string(),
        "DXY" => "DX-Y.NYB".to_string(),
        _ => format!("{}=X", sym),
    };

    let reference_symbol_encoded = reference_symbol.replace('^', "%5E").replace('=', "%3D");
    let Ok(template) = std::env::var("REFERENCE_HISTORY_URL_TEMPLATE") else {
        return Json(json!([]));
    };
    let url = template.replace("{symbol}", &reference_symbol_encoded);

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
                return Json(json!([]));
            }
            match res.json::<Value>().await {
                Ok(json_data) => {
                    if let Some(result) = json_data["chart"]["result"].get(0) {
                        if let (Some(timestamps), Some(closes)) = (
                            result["timestamp"].as_array(),
                            result["indicators"]["quote"][0]["close"].as_array(),
                        ) {
                            let mut history = Vec::new();
                            let len = std::cmp::min(timestamps.len(), closes.len());
                            for i in 0..len {
                                if let (Some(t), Some(c)) =
                                    (timestamps[i].as_i64(), closes[i].as_f64())
                                {
                                    history.push(json!({
                                        "time": t,
                                        "value": c
                                    }));
                                }
                            }
                            let limit = 120;
                            if history.len() > limit {
                                history = history.split_off(history.len() - limit);
                            }
                            return Json(json!(history));
                        }
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

    Json(json!([]))
}
