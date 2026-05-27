use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventEnvelope<T> {
    pub event_id: Uuid,
    pub event_type: String,
    pub schema_version: u16,
    pub occurred_at: DateTime<Utc>,
    pub published_at: DateTime<Utc>,
    pub source: String,
    pub partition_key: String,
    pub metadata: EventMetadata,
    pub payload: T,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct EventMetadata {
    pub tenant_id: Option<String>,
    pub trace: EventTrace,
    pub quality_flags: Vec<String>,
    pub replayed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct EventTrace {
    pub correlation_id: Option<Uuid>,
    pub causation_id: Option<Uuid>,
    pub raw_event_id: Option<Uuid>,
    pub pipeline_version: Option<String>,
}

impl<T> EventEnvelope<T> {
    pub fn new(
        event_type: impl Into<String>,
        source: impl Into<String>,
        partition_key: impl Into<String>,
        payload: T,
    ) -> Self {
        let now = Utc::now();
        Self {
            event_id: Uuid::new_v4(),
            event_type: event_type.into(),
            schema_version: 1,
            occurred_at: now,
            published_at: now,
            source: source.into(),
            partition_key: partition_key.into(),
            metadata: EventMetadata::default(),
            payload,
        }
    }

    pub fn with_trace(mut self, trace: EventTrace) -> Self {
        self.metadata.trace = trace;
        self
    }

    pub fn mark_replayed(mut self) -> Self {
        self.metadata.replayed = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestPayload {
        symbol: String,
    }

    #[test]
    fn creates_versioned_envelope_with_partition_key() {
        let envelope = EventEnvelope::new(
            "md.canonical.ticks.v1",
            "market-data-service",
            "forex:XAUUSD",
            TestPayload {
                symbol: "XAUUSD".to_string(),
            },
        );

        assert_eq!(envelope.event_type, "md.canonical.ticks.v1");
        assert_eq!(envelope.schema_version, 1);
        assert_eq!(envelope.partition_key, "forex:XAUUSD");
        assert!(!envelope.metadata.replayed);
        assert_eq!(envelope.payload.symbol, "XAUUSD");
    }

    #[test]
    fn can_mark_replayed_event() {
        let envelope = EventEnvelope::new("usage.api.requested.v1", "api-gateway", "tenant_a", ())
            .mark_replayed();

        assert!(envelope.metadata.replayed);
    }
}
