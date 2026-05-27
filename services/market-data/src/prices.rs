use crate::clickhouse::LatestPriceTick;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
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

pub async fn list_prices(State(state): State<AppState>) -> Json<Value> {
    let mut prices: Vec<CachedPrice> = state.prices.read().values().cloned().collect();
    if prices.is_empty() {
        prices = load_clickhouse_latest_prices(&state).await;
    }
    if prices.is_empty() {
        prices = load_latest_prices(&state.db).await.unwrap_or_default();
    }

    Json(json!({
        "items": prices.iter().map(|price| price_json_with_calendar(price, Some(&state.calendar))).collect::<Vec<_>>(),
        "total": prices.len(),
    }))
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
        "source": price.source,
        "asset_type": price.asset_type,
        "received_at": price.received_at,
        "session": session,
    })
}

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
        "SELECT symbol, price, bid, ask, volume, source, asset_type, received_at FROM market.market_latest_prices ORDER BY symbol",
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
        "SELECT symbol, price, bid, ask, volume, source, asset_type, received_at FROM market.market_latest_prices WHERE symbol = $1",
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
            },
            None,
        );
        assert_eq!(body["session"]["state"], "open");
    }
}
