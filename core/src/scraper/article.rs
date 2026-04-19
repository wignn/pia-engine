use reqwest::Client;
use scraper::{Html, Selector};
use std::time::Duration;
use tracing::debug;
use url::Url;

use crate::html;

#[derive(Debug, Clone)]
pub struct ScrapedArticle {
    pub url: String,
    pub title: String,
    pub content: String,
    pub author: String,
    pub published_at: String,
    pub image_url: String,
    pub tags: Vec<String>,
    pub word_count: usize,
}

pub struct ArticleScraper {
    client: Client,
}

impl ArticleScraper {
    pub fn new(user_agent: &str, timeout: Duration) -> Self {
        Self {
            client: Client::builder()
                .timeout(timeout)
                .user_agent(user_agent)
                .redirect(reqwest::redirect::Policy::limited(5))
                .pool_max_idle_per_host(3)
                .build()
                .expect("failed to build HTTP client"),
        }
    }

    pub async fn scrape(&self, article_url: &str) -> Result<ScrapedArticle, String> {
        let resp = self
            .client
            .get(article_url)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.5")
            .send()
            .await
            .map_err(|e| format!("fetch page: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("page returned {}", resp.status()));
        }

        let body = resp.text().await.map_err(|e| format!("read body: {}", e))?;
        let doc = Html::parse_document(&body);

        let title = extract_title(&doc);
        let content = extract_content(&doc);

        if title.is_empty() || content.is_empty() {
            return Err("could not extract title or content".into());
        }

        let content = html::clean_content(&content);
        let word_count = content.split_whitespace().count();

        Ok(ScrapedArticle {
            url: article_url.to_string(),
            title,
            content,
            author: extract_author(&doc),
            published_at: extract_date(&doc),
            image_url: extract_image(&doc, article_url),
            tags: extract_tags(&doc),
            word_count,
        })
    }
}

fn extract_title(doc: &Html) -> String {
    let selectors = [
        "article h1",
        "h1.article-title",
        "h1.entry-title",
        "h1.post-title",
        ".article-header h1",
        "h1[itemprop='headline']",
    ];
    for sel_str in &selectors {
        if let Ok(sel) = Selector::parse(sel_str) {
            if let Some(el) = doc.select(&sel).next() {
                let text: String = el.text().collect::<String>().trim().to_string();
                if !text.is_empty() { return text; }
            }
        }
    }
    if let Ok(sel) = Selector::parse("meta[property='og:title']") {
        if let Some(el) = doc.select(&sel).next() {
            if let Some(content) = el.value().attr("content") {
                let text = content.trim().to_string();
                if !text.is_empty() { return text; }
            }
        }
    }
    if let Ok(sel) = Selector::parse("title") {
        if let Some(el) = doc.select(&sel).next() {
            return el.text().collect::<String>().trim().to_string();
        }
    }
    String::new()
}

fn extract_content(doc: &Html) -> String {
    let selectors = [
        "article .content",
        "article .entry-content",
        "article .post-content",
        "article .article-body",
        ".article-content",
        ".story-body",
        "[itemprop='articleBody']",
    ];
    for sel_str in &selectors {
        if let Ok(sel) = Selector::parse(sel_str) {
            if let Some(el) = doc.select(&sel).next() {
                let text: String = el.text().collect::<String>().trim().to_string();
                if text.len() > 200 { return text; }
            }
        }
    }
    // Fallback: article paragraphs
    if let Ok(sel) = Selector::parse("article p") {
        let paragraphs: Vec<String> = doc.select(&sel)
            .map(|el| el.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let joined = paragraphs.join("\n\n");
        if joined.len() > 200 { return joined; }
    }
    // Final fallback: all long paragraphs
    if let Ok(sel) = Selector::parse("p") {
        let paragraphs: Vec<String> = doc.select(&sel)
            .map(|el| el.text().collect::<String>().trim().to_string())
            .filter(|s| s.len() > 50)
            .collect();
        return paragraphs.join("\n\n");
    }
    String::new()
}

fn extract_author(doc: &Html) -> String {
    let selectors = ["[rel='author']", ".author-name", ".byline", "[itemprop='author']"];
    for sel_str in &selectors {
        if let Ok(sel) = Selector::parse(sel_str) {
            if let Some(el) = doc.select(&sel).next() {
                let text: String = el.text().collect::<String>().trim().to_string();
                if !text.is_empty() { return text; }
            }
        }
    }
    if let Ok(sel) = Selector::parse("meta[name='author']") {
        if let Some(el) = doc.select(&sel).next() {
            if let Some(content) = el.value().attr("content") {
                return content.trim().to_string();
            }
        }
    }
    String::new()
}

fn extract_date(doc: &Html) -> String {
    if let Ok(sel) = Selector::parse("time[datetime]") {
        if let Some(el) = doc.select(&sel).next() {
            if let Some(dt) = el.value().attr("datetime") { return dt.to_string(); }
        }
    }
    let meta_sels = [
        "[itemprop='datePublished']",
        "meta[property='article:published_time']",
    ];
    for sel_str in &meta_sels {
        if let Ok(sel) = Selector::parse(sel_str) {
            if let Some(el) = doc.select(&sel).next() {
                if let Some(v) = el.value().attr("datetime").or(el.value().attr("content")) {
                    return v.to_string();
                }
            }
        }
    }
    String::new()
}

fn extract_image(doc: &Html, base_url: &str) -> String {
    if let Ok(sel) = Selector::parse("meta[property='og:image']") {
        if let Some(el) = doc.select(&sel).next() {
            if let Some(content) = el.value().attr("content") {
                return resolve_url(base_url, content);
            }
        }
    }
    let selectors = ["article img", ".article-image img", ".featured-image img"];
    for sel_str in &selectors {
        if let Ok(sel) = Selector::parse(sel_str) {
            if let Some(el) = doc.select(&sel).next() {
                if let Some(src) = el.value().attr("src") {
                    if !src.is_empty() { return resolve_url(base_url, src); }
                }
            }
        }
    }
    String::new()
}

fn extract_tags(doc: &Html) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut tags = Vec::new();

    if let Ok(sel) = Selector::parse("meta[name='keywords']") {
        if let Some(el) = doc.select(&sel).next() {
            if let Some(content) = el.value().attr("content") {
                for k in content.split(',') {
                    let k = k.trim().to_string();
                    if !k.is_empty() && seen.insert(k.clone()) { tags.push(k); }
                }
            }
        }
    }
    if let Ok(sel) = Selector::parse(".tags a, .post-tags a, [rel='tag']") {
        for el in doc.select(&sel) {
            let text: String = el.text().collect::<String>().trim().to_string();
            if !text.is_empty() && seen.insert(text.clone()) { tags.push(text); }
        }
    }
    tags.truncate(10);
    tags
}

fn resolve_url(base: &str, reference: &str) -> String {
    if reference.starts_with("http://") || reference.starts_with("https://") {
        return reference.to_string();
    }
    Url::parse(base)
        .ok()
        .and_then(|b| b.join(reference).ok())
        .map(|u| u.to_string())
        .unwrap_or_else(|| reference.to_string())
}
