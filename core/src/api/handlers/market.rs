use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::{json, Value};

use crate::api::state::AppState;
use crate::ingestion_subscriber;

/// GET /api/v1/market/prices
/// Returns all cached live prices (from all sources: Finnhub, Tiingo, Binance).
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
