use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct ScrapeJob {
    pub id: Option<String>,
    pub url: String,
}

impl ScrapeJob {
    pub fn validate(self, fallback_id: String) -> anyhow::Result<ValidatedJob> {
        let url = self.url.trim().to_owned();
        if url.is_empty() || !(url.starts_with("http://") || url.starts_with("https://")) {
            anyhow::bail!("url must be an absolute http(s) URL");
        }
        Ok(ValidatedJob {
            id: self.id.filter(|id| !id.trim().is_empty()).unwrap_or(fallback_id),
            url,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ValidatedJob {
    pub id: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct News {
    pub title: Option<String>,
    pub author: Option<String>,
    pub published_time: Option<String>,
    pub content: Option<String>,
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct ScrapeResult {
    pub id: String,
    pub url: String,
    pub ok: bool,
    pub news: Option<News>,
    pub error: Option<String>,
}
