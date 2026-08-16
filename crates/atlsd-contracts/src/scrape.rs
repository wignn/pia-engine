use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScrapeJob {
    #[serde(default)]
    pub id: Option<String>,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScrapeResult {
    pub id: String,
    pub url: String,
    pub ok: bool,
    pub news: Option<ScrapedNews>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScrapedNews {
    pub title: Option<String>,
    pub author: Option<String>,
    pub published_time: Option<String>,
    pub content: Option<String>,
    pub url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrape_job_roundtrips_with_optional_id() {
        let job = ScrapeJob {
            id: None,
            url: "https://www.fxstreet.com/news/x".to_string(),
        };
        let json = serde_json::to_string(&job).unwrap();
        let parsed: ScrapeJob = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, job);
        assert!(json.contains("\"url\""));
    }

    #[test]
    fn scrape_result_roundtrips_failure_shape() {
        let result = ScrapeResult {
            id: "abc123".to_string(),
            url: "https://www.investing.com/news/y".to_string(),
            ok: false,
            news: None,
            error: Some("http 403".to_string()),
        };
        let json = serde_json::to_string(&result).unwrap();
        let parsed: ScrapeResult = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, result);
    }
}
