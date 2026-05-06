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
    let mut cache = PRICE_CACHE.write();
    
    let old_price = cache.get(&data.symbol).map(|c| c.price).unwrap_or(data.price);
    let direction = if data.price > old_price {
        "buy".to_string()
    } else if data.price < old_price {
        "sell".to_string()
    } else {
        cache.get(&data.symbol).map(|c| c.direction.clone()).unwrap_or_else(|| "none".to_string())
    };

    let price_str = if data.asset_type == "crypto" {
        format!("{:.2}", data.price)
    } else if data.asset_type == "forex" && data.symbol.contains("JPY") {
        format!("{:.3}", data.price)
    } else if data.asset_type == "forex" {
        format!("{:.5}", data.price)
    } else {
        format!("{:.2}", data.price)
    };

    let cached = CachedPrice {
        symbol: data.symbol.clone(),
        price: data.price,
        price_str,
        direction,
        asset_type: data.asset_type.clone(),
        updated_at: std::time::Instant::now(),
    };

    cache.insert(data.symbol.clone(), cached.clone());
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
    get_price("XAUUSD").map(|p| {
        format!("XAUUSD ${:.2}", p.price)
    })
}
