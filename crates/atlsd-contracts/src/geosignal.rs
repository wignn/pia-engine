use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::news::{EnrichedNewsArticle, ImpactLevel, SentimentLabel};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GeoSignal {
    pub event_id: String,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub source_url: Option<String>,
    pub title: String,
    pub summary: Option<String>,
    pub category: GeoSignalCategory,
    pub country: Option<String>,
    pub region: Option<String>,
    pub location_scope: GeoLocationScope,
    pub severity_score: f64,
    pub sentiment_score: f64,
    pub confidence_score: f64,
    pub affected_assets: Vec<String>,
    pub asset_impact: BTreeMap<String, f64>,
    pub freshness: GeoSignalFreshness,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GeoSignalCategory {
    Macro,
    MarketNews,
    Conflict,
    Trade,
    Energy,
    CountryRisk,
    SupplyChain,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GeoLocationScope {
    Global,
    Region,
    Country,
    Asset,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GeoSignalFreshness {
    Fresh,
    Stale,
    Partial,
}

pub fn severity_from_impact(impact: &ImpactLevel) -> f64 {
    match impact {
        ImpactLevel::Low => 0.25,
        ImpactLevel::Medium => 0.5,
        ImpactLevel::High => 0.75,
        ImpactLevel::Critical => 1.0,
    }
}

pub fn sentiment_score(sentiment: &SentimentLabel) -> f64 {
    match sentiment {
        SentimentLabel::Positive => 1.0,
        SentimentLabel::Negative => -1.0,
        SentimentLabel::Neutral | SentimentLabel::Mixed | SentimentLabel::Unknown => 0.0,
    }
}

impl GeoSignal {
    pub fn from_enriched_news(article: EnrichedNewsArticle) -> Self {
        let severity_score = severity_from_impact(&article.impact_level);
        let timestamp = article.article.published_at.unwrap_or_else(Utc::now);
        let mut asset_impact = BTreeMap::new();

        for symbol in &article.article.symbols {
            let bucket = asset_bucket(symbol);
            asset_impact.insert(bucket.to_string(), severity_score);
        }

        Self {
            event_id: article.article.article_id,
            timestamp,
            source: article.article.source,
            source_url: Some(article.article.url),
            title: article.article.title,
            summary: article.article.summary,
            category: GeoSignalCategory::MarketNews,
            country: None,
            region: None,
            location_scope: GeoLocationScope::Global,
            severity_score,
            sentiment_score: sentiment_score(&article.sentiment),
            confidence_score: if article.relevance_score.is_nan() { 0.0 } else { article.relevance_score },
            affected_assets: article.article.symbols,
            asset_impact,
            freshness: GeoSignalFreshness::Fresh,
        }
    }
}

fn asset_bucket(symbol: &str) -> &'static str {
    let upper = symbol.to_uppercase();
    if matches!(upper.as_str(), "CL" | "BRENT" | "WTI" | "XAUUSD" | "XAGUSD") {
        "commodity"
    } else if upper.contains("USD") || upper.len() == 6 {
        "fx"
    } else {
        "equity"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::news::{EnrichedNewsArticle, ImpactLevel, NewsArticle, SentimentLabel};

    #[test]
    fn maps_impact_level_to_severity_score() {
        assert_eq!(severity_from_impact(&ImpactLevel::Low), 0.25);
        assert_eq!(severity_from_impact(&ImpactLevel::Medium), 0.5);
        assert_eq!(severity_from_impact(&ImpactLevel::High), 0.75);
        assert_eq!(severity_from_impact(&ImpactLevel::Critical), 1.0);
    }

    #[test]
    fn maps_sentiment_label_to_numeric_score() {
        assert_eq!(sentiment_score(&SentimentLabel::Positive), 1.0);
        assert_eq!(sentiment_score(&SentimentLabel::Negative), -1.0);
        assert_eq!(sentiment_score(&SentimentLabel::Neutral), 0.0);
        assert_eq!(sentiment_score(&SentimentLabel::Mixed), 0.0);
        assert_eq!(sentiment_score(&SentimentLabel::Unknown), 0.0);
    }

    #[test]
    fn normalizes_enriched_news_into_market_news_signal() {
        let article = EnrichedNewsArticle {
            article: NewsArticle {
                article_id: "article-1".to_string(),
                source: "Reuters".to_string(),
                title: "Oil jumps as Red Sea tensions disrupt shipping".to_string(),
                url: "https://example.com/oil-red-sea".to_string(),
                summary: Some("Energy markets react to shipping risk.".to_string()),
                published_at: Some(Utc::now()),
                symbols: vec!["CL".to_string(), "USDCAD".to_string()],
                entities: vec!["Red Sea".to_string(), "Saudi Arabia".to_string()],
                language: Some("en".to_string()),
            },
            sentiment: SentimentLabel::Negative,
            impact_level: ImpactLevel::High,
            relevance_score: 0.82,
            model_version: "test-model".to_string(),
        };

        let signal = GeoSignal::from_enriched_news(article);

        assert_eq!(signal.event_id, "article-1");
        assert_eq!(signal.source, "Reuters");
        assert_eq!(signal.category, GeoSignalCategory::MarketNews);
        assert_eq!(signal.location_scope, GeoLocationScope::Global);
        assert_eq!(signal.severity_score, 0.75);
        assert_eq!(signal.sentiment_score, -1.0);
        assert_eq!(signal.confidence_score, 0.82);
        assert_eq!(signal.affected_assets, vec!["CL", "USDCAD"]);
        assert_eq!(signal.asset_impact.get("commodity"), Some(&0.75));
        assert_eq!(signal.asset_impact.get("fx"), Some(&0.75));
        assert_eq!(signal.freshness, GeoSignalFreshness::Fresh);
    }
}
