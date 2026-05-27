use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct ParsedArticle {
    pub title: String,
    pub url: String,
    pub summary: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub content_hash: String,
}

impl ParsedArticle {
    pub fn new(
        title: &str,
        url: &str,
        summary: Option<String>,
        published_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            title: title.to_string(),
            url: url.to_string(),
            summary,
            published_at,
            content_hash: content_hash(url, title),
        }
    }

    pub fn analysis_text(&self) -> String {
        format!(
            "{}\n\n{}",
            self.title,
            self.summary.as_deref().unwrap_or_default()
        )
    }
}

pub fn content_hash(url: &str, title: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(url.trim().as_bytes());
    hasher.update(b"\0");
    hasher.update(title.trim().as_bytes());
    hex::encode(hasher.finalize())
}

pub fn strip_html(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut in_tag = false;
    for ch in value.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => output.push(ch),
            _ => {}
        }
    }
    output
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn parse_rss_date(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc2822(value)
        .or_else(|_| DateTime::parse_from_rfc3339(value))
        .map(|dt| dt.with_timezone(&Utc))
        .ok()
}
