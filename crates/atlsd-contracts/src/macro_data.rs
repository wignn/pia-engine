use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MacroEvent {
    pub event_id: String,
    pub schema_version: u16,
    pub source: String,
    pub feed_type: String,
    pub observed_at: DateTime<Utc>,
    pub published_at: DateTime<Utc>,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "feed_type", rename_all = "snake_case")]
pub enum MacroPayload {
    Rate(MacroRate),
    Spread(MacroSpread),
    Series(MacroSeriesObservation),
    Bond(MacroBondSnapshot),
    NewsScraped(MacroNewsScraped),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MacroNewsScraped {
    pub id: String,
    pub url: String,
    pub title: Option<String>,
    pub author: Option<String>,
    pub published_time: Option<String>,
    pub content: Option<String>,
    pub media_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MacroRate {
    pub country: String,
    pub tenor: String,
    pub date: NaiveDate,
    pub value: f64,
    pub unit: String,
    pub series_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MacroSpread {
    pub country: String,
    pub spread: String,
    pub date: NaiveDate,
    pub value: f64,
    pub series_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MacroSeriesObservation {
    pub series_id: String,
    pub country: String,
    pub title: String,
    pub category: String,
    pub units: Option<String>,
    pub frequency: Option<String>,
    pub date: NaiveDate,
    pub value: f64,
    pub raw_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MacroBondSnapshot {
    pub country: String,
    pub as_of: NaiveDate,
    pub raw: serde_json::Value,
}

impl MacroEvent {
    pub fn decode_payload(&self) -> Result<MacroPayload, serde_json::Error> {
        let mut value = self.payload.clone();
        if let serde_json::Value::Object(object) = &mut value {
            object.insert(
                "feed_type".to_string(),
                serde_json::Value::String(self.feed_type.clone()),
            );
        }
        serde_json::from_value(value)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.event_id.trim().is_empty() {
            return Err("event_id is required".to_string());
        }
        if self.schema_version != 1 {
            return Err(format!(
                "unsupported schema_version {}",
                self.schema_version
            ));
        }
        if self.source.trim().is_empty() || self.feed_type.trim().is_empty() {
            return Err("source and feed_type are required".to_string());
        }

        match self.decode_payload().map_err(|err| err.to_string())? {
            MacroPayload::Rate(rate) => {
                if !rate.value.is_finite()
                    || rate.country.trim().is_empty()
                    || rate.tenor.trim().is_empty()
                {
                    return Err("invalid macro rate".to_string());
                }
            }
            MacroPayload::Spread(spread) => {
                if !spread.value.is_finite()
                    || spread.country.trim().is_empty()
                    || spread.spread.trim().is_empty()
                {
                    return Err("invalid macro spread".to_string());
                }
            }
            MacroPayload::Series(series) => {
                if !series.value.is_finite()
                    || series.series_id.trim().is_empty()
                    || series.country.trim().is_empty()
                {
                    return Err("invalid macro series".to_string());
                }
            }
            MacroPayload::Bond(bond) => {
                if bond.country.trim().is_empty() {
                    return Err("invalid macro bond snapshot".to_string());
                }
            }
            MacroPayload::NewsScraped(news) => {
                if news.id.trim().is_empty() {
                    return Err("invalid news scraped id".to_string());
                }
            }
        }
        Ok(())
    }
}

pub fn published_now() -> DateTime<Utc> {
    Utc::now()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_rate_event() {
        let event = MacroEvent {
            event_id: "rate-1".to_string(),
            schema_version: 1,
            source: "fred".to_string(),
            feed_type: "rate".to_string(),
            observed_at: Utc::now(),
            published_at: Utc::now(),
            payload: serde_json::json!({
                "country": "US",
                "tenor": "10Y",
                "date": "2026-08-24",
                "value": 4.2,
                "unit": "percent",
                "series_id": "DGS10"
            }),
        };
        assert!(event.validate().is_ok());
    }
}
