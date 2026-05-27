use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WhyMoveExplanation {
    pub symbol: String,
    pub window: String,
    pub price_move_pct: f64,
    pub primary_drivers: Vec<FactorScore>,
    pub confidence: ConfidenceScore,
    pub narrative: String,
    pub evidence_ids: Vec<String>,
    pub generated_at: DateTime<Utc>,
    pub pipeline_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FactorScore {
    pub factor: String,
    pub score: i16,
    pub confidence: f64,
    pub lookback: String,
    pub evidence_ids: Vec<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FearGreedScore {
    pub scope: String,
    pub score: u8,
    pub label: FearGreedLabel,
    pub factors: Vec<FactorScore>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConfidenceScore {
    pub score: f64,
    pub label: ConfidenceLabel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConfidenceLabel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FearGreedLabel {
    ExtremeFear,
    Fear,
    Neutral,
    Greed,
    ExtremeGreed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_envelope::EventEnvelope;
    use crate::topics;

    #[test]
    fn why_move_explanation_keeps_evidence_lineage() {
        let now = Utc::now();
        let explanation = WhyMoveExplanation {
            symbol: "XAUUSD".to_string(),
            window: "15m".to_string(),
            price_move_pct: 0.42,
            primary_drivers: vec![FactorScore {
                factor: "usd_pressure".to_string(),
                score: -72,
                confidence: 0.81,
                lookback: "30m".to_string(),
                evidence_ids: vec!["news_1".to_string(), "dxy_move_2".to_string()],
                updated_at: now,
            }],
            confidence: ConfidenceScore {
                score: 0.78,
                label: ConfidenceLabel::High,
            },
            narrative: "Gold moved higher as USD weakness supported defensive demand.".to_string(),
            evidence_ids: vec!["news_1".to_string(), "candle_1".to_string()],
            generated_at: now,
            pipeline_version: "why-move@2.1.0".to_string(),
        };

        let envelope = EventEnvelope::new(
            topics::INTELLIGENCE_WHY_MOVE_GENERATED_V1,
            "intelligence-service",
            topics::market_partition_key("commodity", &explanation.symbol),
            explanation,
        );

        assert_eq!(envelope.payload.evidence_ids.len(), 2);
        assert_eq!(envelope.payload.primary_drivers[0].factor, "usd_pressure");
    }
}
