use crate::clickhouse::LatestPriceTick;
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderName, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedPrice {
    pub symbol: String,
    pub price: f64,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub volume: Option<f64>,
    pub source: String,
    pub asset_type: String,
    pub received_at: Option<String>,
    pub timestamp_ms: Option<i64>,
    pub feed: Option<String>,
}

type LatestPriceRow = (
    String,
    f64,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    String,
    String,
    Option<chrono::DateTime<chrono::Utc>>,
    Option<i64>,
);

pub async fn hydrate_price_cache(state: &AppState) -> anyhow::Result<usize> {
    let rows = load_latest_prices(&state.db).await?;
    let count = rows.len();
    let mut cache = state.prices.write();
    for price in rows {
        cache.insert(price.symbol.clone(), price);
    }
    Ok(count)
}

pub async fn compute_prices_snapshot_bytes(state: &AppState) -> (axum::body::Bytes, &'static str) {
    if let Some(bytes) = {
        let guard = state.snapshot_cache.read();
        guard.as_ref().and_then(|(cached_at, b)| {
            if cached_at.elapsed() < std::time::Duration::from_millis(100) {
                Some(b.clone())
            } else {
                None
            }
        })
    } {
        return (bytes, "HIT");
    }

    let mut prices: Vec<CachedPrice> = state.prices.read().values().cloned().collect();
    if prices.is_empty() {
        prices = load_clickhouse_latest_prices(state).await;
    }
    if prices.is_empty() {
        prices = load_latest_prices(&state.db).await.unwrap_or_default();
    }

    let payload = json!({
        "items": prices.iter().map(|price| price_json_with_calendar(price, Some(&state.calendar))).collect::<Vec<_>>(),
        "total": prices.len(),
    });

    let bytes = axum::body::Bytes::from(serde_json::to_vec(&payload).unwrap_or_default());
    {
        let mut guard = state.snapshot_cache.write();
        *guard = Some((std::time::Instant::now(), bytes.clone()));
    }
    (bytes, "MISS")
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ListSymbolsParams {
    pub asset_type: Option<String>,
    pub search: Option<String>,
    pub exchange: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

pub async fn list_symbols(
    State(state): State<AppState>,
    Query(params): Query<ListSymbolsParams>,
) -> Response {
    let mut prices: Vec<CachedPrice> = state.prices.read().values().cloned().collect();
    if prices.is_empty() {
        prices = load_clickhouse_latest_prices(&state).await;
    }
    if prices.is_empty() {
        prices = load_latest_prices(&state.db).await.unwrap_or_default();
    }

    let mut items: Vec<Value> = prices
        .into_iter()
        .filter(|p| {
            if let Some(ref at) = params.asset_type {
                if !p.asset_type.eq_ignore_ascii_case(at) {
                    return false;
                }
            }
            if let Some(ref ex) = params.exchange {
                if !p.source.eq_ignore_ascii_case(ex) {
                    return false;
                }
            }
            if let Some(ref q) = params.search {
                let q_upper = q.to_uppercase();
                if !p.symbol.contains(&q_upper) {
                    return false;
                }
            }
            true
        })
        .map(|p| {
            let exchange_meta = state.calendar.exchange_for_symbol(&p.symbol);
            let exchange_code = exchange_meta
                .as_ref()
                .map(|e| e.exchange_code.clone())
                .unwrap_or_else(|| p.source.to_uppercase());
            let (decimals, tick_size) = default_precision_and_tick(&p.symbol, &p.asset_type);

            json!({
                "symbol": p.symbol,
                "asset_type": p.asset_type,
                "exchange": exchange_code,
                "source": p.source,
                "price_precision": decimals,
                "tick_size": tick_size,
                "is_active": true,
                "last_price": p.price,
            })
        })
        .collect();

    items.sort_by(|a, b| {
        let sym_a = a.get("symbol").and_then(|v| v.as_str()).unwrap_or("");
        let sym_b = b.get("symbol").and_then(|v| v.as_str()).unwrap_or("");
        sym_a.cmp(sym_b)
    });

    let total = items.len();
    let offset = params.offset.unwrap_or(0).min(total);
    let limit = params.limit.unwrap_or(total.max(1)).clamp(1, 1000);
    let items = items
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect::<Vec<_>>();
    let next_offset = offset + items.len();
    let payload = json!({
        "total": total,
        "items": items,
        "offset": offset,
        "limit": limit,
        "next_offset": (next_offset < total).then_some(next_offset),
    });

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(payload),
    )
        .into_response()
}

fn default_precision_and_tick(symbol: &str, asset_type: &str) -> (u32, f64) {
    match asset_type.to_lowercase().as_str() {
        "forex" => {
            if symbol.ends_with("JPY") {
                (3, 0.001)
            } else {
                (5, 0.00001)
            }
        }
        "crypto" => {
            if symbol.starts_with("BTC") || symbol.starts_with("ETH") {
                (2, 0.01)
            } else {
                (4, 0.0001)
            }
        }
        "commodity" => (3, 0.001),
        "stock" => (2, 0.01),
        "rates" => (3, 0.001),
        _ => (2, 0.01),
    }
}

pub async fn list_prices(State(state): State<AppState>) -> Response {
    let (bytes, cache_status) = compute_prices_snapshot_bytes(&state).await;
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/json"),
            (HeaderName::from_static("x-cache"), cache_status),
        ],
        axum::body::Body::from(bytes),
    )
        .into_response()
}

pub async fn get_price(Path(symbol): Path<String>, State(state): State<AppState>) -> Json<Value> {
    let symbol = symbol.to_uppercase();
    let cached = { state.prices.read().get(&symbol).cloned() };
    let price = if cached.is_some() {
        cached
    } else if let Some(price) = load_clickhouse_latest_price(&state, &symbol).await {
        Some(price)
    } else {
        load_latest_price(&state.db, &symbol).await.ok().flatten()
    };

    match price {
        Some(price) => Json(price_json_with_calendar(&price, Some(&state.calendar))),
        None => Json(json!({ "symbol": symbol, "error": "price not found" })),
    }
}

pub async fn get_orderbook(Path(symbol): Path<String>, State(state): State<AppState>) -> Response {
    let symbol = symbol.trim().to_uppercase();

    // Quote-derived synthetic depth keeps the panel populated for instruments
    // without a native Level 2 feed. It is explicitly labeled in the response.
    let cached = { state.prices.read().get(&symbol).cloned() };
    let price = if cached.is_some() {
        cached
    } else if let Some(price) = load_clickhouse_latest_price(&state, &symbol).await {
        Some(price)
    } else {
        load_latest_price(&state.db, &symbol).await.ok().flatten()
    };

    let p = match price {
        Some(p) => p,
        None => {
            return Json(json!({
                "symbol": symbol,
                "bids": [],
                "asks": [],
                "timestamp": chrono::Utc::now().timestamp_millis(),
                "error": "symbol not found"
            }))
            .into_response();
        }
    };

    let (decimals, tick_size) = default_precision_and_tick(&p.symbol, &p.asset_type);
    let factor = 10f64.powi(decimals as i32);
    let mid_price = p.price;
    let base_bid = p.bid.unwrap_or(mid_price - tick_size);
    let base_ask = p.ask.unwrap_or(mid_price + tick_size);

    let mut bids = Vec::new();
    let mut asks = Vec::new();

    let base_vol = p.volume.unwrap_or(100.0).max(1.0);

    for i in 0..15 {
        let step = (i as f64) * tick_size;
        let bid_p = ((base_bid - step) * factor).round() / factor;
        let ask_p = ((base_ask + step) * factor).round() / factor;

        let bid_sz = (((base_vol * (0.15 + (i as f64) * 0.08)) * 100.0).round() / 100.0).max(0.01);
        let ask_sz = (((base_vol * (0.14 + (i as f64) * 0.075)) * 100.0).round() / 100.0).max(0.01);

        if bid_p > 0.0 {
            bids.push(json!({ "price": bid_p, "size": bid_sz }));
        }
        if ask_p > 0.0 {
            asks.push(json!({ "price": ask_p, "size": ask_sz }));
        }
    }

    Json(json!({
        "symbol": p.symbol,
        "bids": bids,
        "asks": asks,
        "timestamp": p.timestamp_ms.unwrap_or_else(|| chrono::Utc::now().timestamp_millis()),
        "source": p.source,
        "is_live": false,
        "synthetic": true,
        "depth_type": "quote_derived",
        "unavailable_reason": "Synthetic depth: generated from latest bid/ask and quote volume"
    }))
    .into_response()
}

pub fn price_json_with_calendar(
    price: &CachedPrice,
    calendar: Option<&crate::calendar::CalendarCache>,
) -> Value {
    let session = crate::session::session_status(
        &price.symbol,
        &price.asset_type,
        chrono::Utc::now(),
        calendar,
    );
    json!({
        "symbol": price.symbol,
        "price": price.price,
        "bid": price.bid,
        "ask": price.ask,
        "volume": price.volume,
        "volume_type": if price.volume.is_some() {
            if price.feed.as_deref() == Some("crypto") || price.feed.as_deref() == Some("stock") {
                "exchange"
            } else {
                "tick"
            }
        } else {
            "unavailable"
        },
        "volume_available": price.volume.is_some(),
        "source": price.source,
        "asset_type": price.asset_type,
        "received_at": price.received_at,
        "timestamp_ms": price.timestamp_ms,
        "feed": price.feed,
        "session": session,
    })
}

fn cached_price_from_row(row: LatestPriceRow) -> CachedPrice {
    let (symbol, price, bid, ask, volume, source, asset_type, received_at, provider_ts_ms) = row;
    CachedPrice {
        symbol,
        price,
        bid,
        ask,
        volume,
        source,
        asset_type,
        received_at: received_at.map(|dt| dt.to_rfc3339()),
        timestamp_ms: provider_ts_ms,
        feed: None,
    }
}

fn cached_price_from_clickhouse(row: LatestPriceTick) -> CachedPrice {
    CachedPrice {
        symbol: row.symbol,
        price: row.price,
        bid: row.bid,
        ask: row.ask,
        volume: Some(row.volume),
        source: row.source,
        asset_type: row.asset_type,
        received_at: Some(row.received_at),
        timestamp_ms: None,
        feed: None,
    }
}

async fn load_clickhouse_latest_prices(state: &AppState) -> Vec<CachedPrice> {
    let Some(clickhouse) = &state.clickhouse else {
        return Vec::new();
    };
    match clickhouse.latest_prices().await {
        Ok(rows) => rows.into_iter().map(cached_price_from_clickhouse).collect(),
        Err(err) => {
            tracing::warn!(error = %err, "failed to load latest market prices from ClickHouse");
            Vec::new()
        }
    }
}

async fn load_clickhouse_latest_price(state: &AppState, symbol: &str) -> Option<CachedPrice> {
    let clickhouse = state.clickhouse.as_ref()?;
    match clickhouse.latest_price(symbol).await {
        Ok(Some(row)) => Some(cached_price_from_clickhouse(row)),
        Ok(None) => None,
        Err(err) => {
            tracing::warn!(error = %err, symbol = %symbol, "failed to load latest market price from ClickHouse");
            None
        }
    }
}

pub async fn load_latest_prices(pool: &sqlx::PgPool) -> Result<Vec<CachedPrice>, sqlx::Error> {
    let rows: Vec<LatestPriceRow> = sqlx::query_as(
        "SELECT symbol, price, bid, ask, volume, source, asset_type, received_at, provider_ts_ms FROM market.market_latest_prices ORDER BY symbol",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(cached_price_from_row).collect())
}

pub async fn load_latest_price(
    pool: &sqlx::PgPool,
    symbol: &str,
) -> Result<Option<CachedPrice>, sqlx::Error> {
    let row: Option<LatestPriceRow> = sqlx::query_as(
        "SELECT symbol, price, bid, ask, volume, source, asset_type, received_at, provider_ts_ms FROM market.market_latest_prices WHERE symbol = $1",
    )
    .bind(symbol)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(cached_price_from_row))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn price_json_includes_session_metadata() {
        let body = price_json_with_calendar(
            &CachedPrice {
                symbol: "BTCUSDT".to_string(),
                price: 100.0,
                bid: None,
                ask: None,
                volume: Some(1.0),
                source: "test".to_string(),
                asset_type: "crypto".to_string(),
                received_at: None,
                timestamp_ms: None,
                feed: None,
            },
            None,
        );
        assert_eq!(body["session"]["state"], "open");
    }
}
