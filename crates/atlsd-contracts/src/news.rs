use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NewsArticle {
    pub article_id: String,
    pub source: String,
    pub title: String,
    pub url: String,
    pub summary: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub symbols: Vec<String>,
    pub entities: Vec<String>,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnrichedNewsArticle {
    pub article: NewsArticle,
    pub sentiment: SentimentLabel,
    pub impact_level: ImpactLevel,
    pub relevance_score: f64,
    pub model_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SentimentLabel {
    Positive,
    Negative,
    Neutral,
    Mixed,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ImpactLevel {
    Low,
    Medium,
    High,
    Critical,
}
