use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarketTick {
    pub symbol: String,
    pub asset_class: AssetClass,
    pub venue: String,
    pub price: f64,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub volume: Option<f64>,
    pub source_sequence: Option<String>,
    pub ts_exchange: Option<DateTime<Utc>>,
    pub ts_received: DateTime<Utc>,
    pub ts_normalized: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OhlcvCandle {
    pub symbol: String,
    pub asset_class: AssetClass,
    pub resolution: CandleResolution,
    pub bucket_start: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub tick_count: u64,
    pub corrected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarketDataQualityEvent {
    pub symbol: String,
    pub source: String,
    pub status: DataQualityStatus,
    pub score: f64,
    pub reason: String,
    pub observed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AssetClass {
    Forex,
    Equity,
    Index,
    Crypto,
    Commodity,
    Rates,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CandleResolution {
    OneSecond,
    OneMinute,
    FiveMinutes,
    FifteenMinutes,
    OneHour,
    OneDay,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DataQualityStatus {
    Ok,
    Stale,
    Flat,
    Gap,
    Outlier,
    Duplicate,
    Invalid,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_envelope::EventEnvelope;
    use crate::topics;

    #[test]
    fn market_tick_can_be_wrapped_in_canonical_topic() {
        let now = Utc::now();
        let tick = MarketTick {
            symbol: "XAUUSD".to_string(),
            asset_class: AssetClass::Commodity,
            venue: "tiingo".to_string(),
            price: 2368.42,
            bid: Some(2368.40),
            ask: Some(2368.44),
            volume: Some(10.0),
            source_sequence: Some("seq-1".to_string()),
            ts_exchange: Some(now),
            ts_received: now,
            ts_normalized: now,
        };

        let envelope = EventEnvelope::new(
            topics::MD_CANONICAL_TICKS_V1,
            "market-data-service",
            topics::market_partition_key("commodity", &tick.symbol),
            tick,
        );

        assert_eq!(envelope.event_type, topics::MD_CANONICAL_TICKS_V1);
        assert_eq!(envelope.partition_key, "commodity:XAUUSD");
    }
}
