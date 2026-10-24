use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::event_envelope::EventEnvelope;

/// Versioned feed payload accepted by the market-data ingestion service.
/// Provider-specific fields stay optional so feeds can share this contract.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct RawMarketTick {
    pub feed: String,
    pub source: String,
    pub symbol: String,
    pub asset_type: Option<String>,
    pub price: f64,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub volume: Option<f64>,
    pub quantity: Option<f64>,
    pub timestamp_ms: Option<i64>,
    pub trade_time_ms: Option<i64>,
    pub received_at: Option<String>,
    pub is_buyer_maker: Option<bool>,
}

impl Default for RawMarketTick {
    fn default() -> Self {
        Self {
            feed: String::new(),
            source: String::new(),
            symbol: String::new(),
            asset_type: None,
            price: 0.0,
            bid: None,
            ask: None,
            volume: None,
            quantity: None,
            timestamp_ms: None,
            trade_time_ms: None,
            received_at: None,
            is_buyer_maker: None,
        }
    }
}

impl RawMarketTick {
    pub fn into_envelope(
        self,
        event_type: impl Into<String>,
        event_id: Uuid,
    ) -> EventEnvelope<Self> {
        let partition_key = format!("{}:{}", self.feed, self.symbol.to_uppercase());
        let source = if self.source.is_empty() {
            self.feed.clone()
        } else {
            self.source.clone()
        };
        EventEnvelope::new(event_type, source, partition_key, self).with_event_id(event_id)
    }
}

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
    pub sequence: u64,
    pub closed_at: DateTime<Utc>,
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

    #[test]
    fn raw_market_tick_envelope_round_trips_with_stable_id() {
        let tick = RawMarketTick {
            feed: "crypto".to_string(),
            source: "binance".to_string(),
            symbol: "BTCUSDT".to_string(),
            price: 67_000.0,
            timestamp_ms: Some(1_710_000_000_000),
            ..Default::default()
        };
        let id = crate::event_envelope::stable_event_id("binance:BTCUSDT:1710000000000");
        let envelope = tick.into_envelope(topics::MD_RAW_CRYPTO_TRADES_V1, id);
        let encoded = serde_json::to_string(&envelope).unwrap();
        let decoded: EventEnvelope<RawMarketTick> = serde_json::from_str(&encoded).unwrap();

        assert_eq!(decoded.event_id, id);
        assert_eq!(decoded.event_type, topics::MD_RAW_CRYPTO_TRADES_V1);
        assert_eq!(decoded.partition_key, "crypto:BTCUSDT");
        assert_eq!(decoded.payload.price, 67_000.0);
    }
}
