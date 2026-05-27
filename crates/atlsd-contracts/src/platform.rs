use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EntitlementChanged {
    pub tenant_id: String,
    pub plan_id: String,
    pub allowed_symbols: Vec<String>,
    pub allowed_channels: Vec<String>,
    pub max_ws_connections: u32,
    pub history_depth_days: u32,
    pub can_use_llm: bool,
    pub changed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UsageEvent {
    pub tenant_id: String,
    pub api_key_id: Option<String>,
    pub usage_kind: UsageKind,
    pub quantity: u64,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UsageKind {
    ApiRequest { route: String, method: String },
    WebSocketConnected { channel_count: u32 },
    WebSocketMessageDelivered { channel: String },
    HistoricalCandlesQueried { symbol: String, resolution: String },
    IntelligenceGenerated { feature: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditEvent {
    pub actor_id: String,
    pub tenant_id: Option<String>,
    pub action: String,
    pub target: String,
    pub occurred_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_envelope::EventEnvelope;
    use crate::topics;

    #[test]
    fn entitlement_event_uses_tenant_partition() {
        let event = EntitlementChanged {
            tenant_id: "tenant_a".to_string(),
            plan_id: "enterprise".to_string(),
            allowed_symbols: vec!["XAUUSD".to_string()],
            allowed_channels: vec!["market.price".to_string(), "why_move".to_string()],
            max_ws_connections: 25,
            history_depth_days: 3650,
            can_use_llm: true,
            changed_at: Utc::now(),
        };

        let envelope = EventEnvelope::new(
            topics::TENANT_ENTITLEMENT_CHANGED_V1,
            "control-plane",
            topics::tenant_partition_key(&event.tenant_id),
            event,
        );

        assert_eq!(envelope.partition_key, "tenant_a");
        assert!(envelope.payload.can_use_llm);
    }
}
