use serde::{Deserialize, Serialize};
use tracing::debug;

use super::text::ParsedArticle;

#[derive(Debug, Clone)]
pub struct ArticleAnalysis {
    pub sentiment: String,
    pub impact_level: String,
    pub currency_pairs: String,
}

#[derive(Clone)]
pub struct AnalyzerClient {
    client: reqwest::Client,
    base_url: Option<String>,
}

#[derive(Serialize)]
struct AnalyzeRequest<'a> {
    text: &'a str,
}

#[derive(Deserialize)]
struct AnalyzeResponse {
    sentiment: Option<String>,
    entities: Option<Entities>,
    event: Option<Event>,
}

#[derive(Deserialize)]
struct Entities {
    currencies: Vec<String>,
}

#[derive(Deserialize)]
struct Event {
    #[serde(rename = "type")]
    event_type: Option<String>,
}

impl AnalyzerClient {
    pub fn new(client: reqwest::Client, base_url: Option<String>) -> Self {
        Self { client, base_url }
    }

    pub async fn analyze(&self, article: &ParsedArticle) -> ArticleAnalysis {
        let text = article.analysis_text();
        if let Some(base_url) = &self.base_url {
            match self.analyze_remote(base_url, &text).await {
                Ok(analysis) => return analysis,
                Err(err) => debug!(error = %err, "remote analyzer failed, using local fallback"),
            }
        }

        ArticleAnalysis {
            sentiment: detect_sentiment(&text).to_string(),
            impact_level: detect_impact_level(&text).to_string(),
            currency_pairs: detect_currency_pairs(&text).join(", "),
        }
    }

    async fn analyze_remote(&self, base_url: &str, text: &str) -> anyhow::Result<ArticleAnalysis> {
        let endpoint = format!("{}/analyze", base_url.trim_end_matches('/'));
        let response = self
            .client
            .post(endpoint)
            .json(&AnalyzeRequest { text })
            .send()
            .await?
            .error_for_status()?
            .json::<AnalyzeResponse>()
            .await?;

        let currency_pairs = response
            .entities
            .map(|entities| entities.currencies.join(", "))
            .unwrap_or_else(|| detect_currency_pairs(text).join(", "));

        Ok(ArticleAnalysis {
            sentiment: response
                .sentiment
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| detect_sentiment(text).to_string()),
            impact_level: response
                .event
                .and_then(|event| event.event_type)
                .map(|_| "high".to_string())
                .unwrap_or_else(|| detect_impact_level(text).to_string()),
            currency_pairs,
        })
    }
}

fn detect_currency_pairs(text: &str) -> Vec<&'static str> {
    const PAIRS: &[&str] = &[
        "EURUSD", "GBPUSD", "USDJPY", "AUDUSD", "USDCAD", "USDCHF", "NZDUSD", "EURJPY", "GBPJPY",
        "AUDJPY", "XAUUSD", "DXY",
    ];
    let normalized = text.to_uppercase().replace(['/', '-'], "");
    PAIRS
        .iter()
        .copied()
        .filter(|pair| normalized.contains(pair))
        .collect()
}

fn detect_impact_level(text: &str) -> &'static str {
    let lower = text.to_lowercase();
    if [
        "fed",
        "fomc",
        "ecb",
        "boj",
        "boe",
        "inflation",
        "cpi",
        "nfp",
        "payroll",
        "rate decision",
        "interest rate",
        "jobs report",
    ]
    .iter()
    .any(|term| lower.contains(term))
    {
        "high"
    } else if ["gdp", "pmi", "retail sales", "unemployment", "yield"]
        .iter()
        .any(|term| lower.contains(term))
    {
        "medium"
    } else {
        "low"
    }
}

fn detect_sentiment(text: &str) -> &'static str {
    let lower = text.to_lowercase();
    let positive = [
        "rally", "gain", "rise", "surge", "bull", "optimism", "rebound",
    ];
    let negative = [
        "fall", "drop", "slump", "bear", "risk-off", "concern", "weak", "decline",
    ];
    let pos = positive
        .iter()
        .filter(|term| lower.contains(**term))
        .count();
    let neg = negative
        .iter()
        .filter(|term| lower.contains(**term))
        .count();

    match pos.cmp(&neg) {
        std::cmp::Ordering::Greater => "positive",
        std::cmp::Ordering::Less => "negative",
        std::cmp::Ordering::Equal => "neutral",
    }
}
