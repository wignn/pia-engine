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
                "source": p.source,
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

/// GET /api/v1/market/history/{symbol}
/// Proxies and normalizes historical candle data from Binance or Yahoo Finance.
pub async fn get_history(_state: State<AppState>, Path(symbol): Path<String>) -> Json<Value> {
    let sym = symbol.to_uppercase();

    // 1. Binance Crypto
    if sym.ends_with("USDT") {
        let url = format!(
            "https://api.binance.com/api/v3/klines?symbol={}&interval=1m&limit=120",
            sym
        );
        match HTTP_CLIENT.get(&url).send().await {
            Ok(res) => {
                if let Ok(data) = res.json::<Vec<Vec<Value>>>().await {
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
            }
            Err(e) => {
                tracing::warn!("Failed to fetch Binance history for {}: {:?}", sym, e);
            }
        }
    }

    // 2. Yahoo Finance (Forex, Indices, Commodities)
    let yahoo_symbol = match sym.as_str() {
        "XAUUSD" => "GC=F".to_string(),
        "SPX" => "^GSPC".to_string(),
        "DXY" => "DX-Y.NYB".to_string(),
        _ => format!("{}=X", sym),
    };

    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{}?interval=1m&range=1d",
        yahoo_symbol
    );

    match HTTP_CLIENT.get(&url).send().await {
        Ok(res) => {
            if let Ok(json_data) = res.json::<Value>().await {
                if let Some(result) = json_data["chart"]["result"].get(0) {
                    if let (Some(timestamps), Some(closes)) = (
                        result["timestamp"].as_array(),
                        result["indicators"]["quote"][0]["close"].as_array(),
                    ) {
                        let mut history = Vec::new();
                        let len = std::cmp::min(timestamps.len(), closes.len());
                        for i in 0..len {
                            if let (Some(t), Some(c)) = (timestamps[i].as_i64(), closes[i].as_f64())
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
            }
        }
        Err(e) => {
            tracing::warn!("Failed to fetch Yahoo history for {}: {:?}", sym, e);
        }
    }

    Json(json!([]))
}
