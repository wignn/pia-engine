use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Deserialize)]
pub struct MarketTradeEvent {
    pub event: String,
    pub data: Option<MarketTradeDataWrapper>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MarketTradeDataWrapper {
    pub tick: MarketTradeData,
    pub asset_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MarketTradeData {
    pub symbol: String,
    pub price: f64,
    pub asset_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MarketTickData {
    pub symbol: String,
    pub price: f64,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
pub struct CachedPrice {
    pub symbol: String,
    pub price: f64,
    pub price_str: String,
    pub direction: String,
    pub asset_type: String,
    pub updated_at: std::time::Instant,
}

static PRICE_CACHE: Lazy<Arc<RwLock<HashMap<String, CachedPrice>>>> =
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

pub fn update_price(data: &MarketTradeData) -> CachedPrice {
    update_cached_price(&data.symbol, data.price, &data.asset_type)
}

pub fn update_tick(data: &MarketTickData) -> CachedPrice {
    let asset_type = infer_asset_type(&data.symbol);
    update_cached_price(&data.symbol, data.price, asset_type)
}

fn update_cached_price(symbol: &str, price: f64, asset_type: &str) -> CachedPrice {
    let mut cache = PRICE_CACHE.write();

    let old_price = cache.get(symbol).map(|c| c.price).unwrap_or(price);
    let direction = if price > old_price {
        "buy".to_string()
    } else if price < old_price {
        "sell".to_string()
    } else {
        cache
            .get(symbol)
            .map(|c| c.direction.clone())
            .unwrap_or_else(|| "none".to_string())
    };

    let price_str = if asset_type == "crypto" {
        format!("{price:.2}")
    } else if asset_type == "forex" && symbol.contains("JPY") {
        format!("{price:.3}")
    } else if asset_type == "forex" {
        format!("{price:.5}")
    } else {
        format!("{price:.2}")
    };

    let cached = CachedPrice {
        symbol: symbol.to_string(),
        price,
        price_str,
        direction,
        asset_type: asset_type.to_string(),
        updated_at: std::time::Instant::now(),
    };

    cache.insert(symbol.to_string(), cached.clone());
    cached
}

pub fn get_price(symbol: &str) -> Option<CachedPrice> {
    let cache = PRICE_CACHE.read();
    let upper = symbol.to_uppercase();
    cache.get(&upper).cloned()
}

pub fn get_all_prices() -> Vec<CachedPrice> {
    let cache = PRICE_CACHE.read();
    cache.values().cloned().collect()
}

pub fn get_xauusd_display() -> Option<String> {
    get_price("XAUUSD").map(|p| format!("XAUUSD ${:.2}", p.price))
}

fn infer_asset_type(symbol: &str) -> &'static str {
    if symbol.ends_with("USDT") || symbol.ends_with("BTC") || symbol.ends_with("ETH") {
        "crypto"
    } else if symbol.len() == 6 && !symbol.starts_with("XAU") && !symbol.starts_with("XAG") {
        "forex"
    } else {
        "unknown"
    }
}

#[cfg(test)]
mod tests {
    use super::{MarketTickData, update_tick};

    #[test]
    fn parses_new_tick_and_formats_price() {
        let tick: MarketTickData = serde_json::from_str(
            r#"{"symbol":"EURUSD","price":1.082567,"bid":1.0825,"ask":1.0826,"timestamp":"2026-08-11T12:00:00Z"}"#,
        )
        .unwrap();
        let cached = update_tick(&tick);
        assert_eq!(cached.asset_type, "forex");
        assert_eq!(cached.price_str, "1.08257");
    }
}
