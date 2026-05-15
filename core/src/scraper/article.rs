use reqwest::Client;
use scraper::{ElementRef, Html, Selector};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
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
    max_retries: usize,
}

impl ArticleScraper {
    pub fn new(user_agent: &str, timeout: Duration) -> Self {
        Self {
            client: Client::builder()
                .timeout(timeout)
                .connect_timeout(timeout.min(Duration::from_secs(15)))
                .user_agent(user_agent)
                .redirect(reqwest::redirect::Policy::custom(|attempt| {
                    if attempt.previous().len() >= 5 {
                        attempt.error("too many redirects")
                    } else if validate_url(attempt.url()).is_err() {
                        attempt.error("redirect target is not allowed")
                    } else {
                        attempt.follow()
                    }
                }))
                .pool_max_idle_per_host(8)
                .pool_idle_timeout(Duration::from_secs(90))
                .tcp_keepalive(Some(Duration::from_secs(60)))
                .build()
                .expect("failed to build HTTP client"),
            max_retries: 3,
        }
    }

    pub async fn scrape(&self, article_url: &str) -> Result<ScrapedArticle, String> {
        let parsed_url = validate_fetch_url(article_url)?;
        validate_resolved_host(&parsed_url).await?;

        let body = self.fetch_html(article_url).await?;
        let doc = Html::parse_document(&body);

        let title = extract_title(&doc);
        let content = extract_content(&doc);

        if content.is_empty() {
            return Err("could not extract article content".into());
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

    async fn fetch_html(&self, article_url: &str) -> Result<String, String> {
        let mut last_error = String::new();

        for attempt in 1..=self.max_retries {
            let resp = self
                .client
                .get(article_url)
                .header(
                    "Accept",
                    "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
                )
                .header("Accept-Language", "en-US,en;q=0.9,id;q=0.8")
                .header("Cache-Control", "no-cache")
                .header("Pragma", "no-cache")
                .header("Upgrade-Insecure-Requests", "1")
                .send()
                .await;

            match resp {
                Ok(resp) => {
                    let status = resp.status();
                    validate_url(resp.url())?;
                    if !status.is_success() {
                        let msg = format!("page returned {}", status);
                        if !is_retryable_status(status.as_u16()) {
                            return Err(msg);
                        }
                        last_error = msg;
                    } else {
                        match resp.text().await {
                            Ok(body) => return Ok(body),
                            Err(e) => last_error = format!("read body: {}", e),
                        }
                    }
                }
                Err(e) => {
                    last_error = format!("fetch page: {}", e);
                }
            }

            if attempt < self.max_retries {
                let backoff = Duration::from_millis(300 * attempt as u64);
                debug!(
                    url = %article_url,
                    attempt,
                    error = %last_error,
                    backoff_ms = backoff.as_millis(),
                    "article fetch failed, retrying"
                );
                tokio::time::sleep(backoff).await;
            }
        }

        Err(last_error)
    }
}

fn validate_fetch_url(article_url: &str) -> Result<Url, String> {
    let parsed_url = Url::parse(article_url).map_err(|e| format!("invalid url: {}", e))?;
    validate_url(&parsed_url)?;
    Ok(parsed_url)
}

fn validate_url(url: &Url) -> Result<(), String> {
    if !matches!(url.scheme(), "http" | "https") {
        return Err(format!("unsupported url scheme: {}", url.scheme()));
    }

    let Some(host) = url.host_str() else {
        return Err("url host is required".into());
    };

    if is_blocked_host(host) {
        return Err("url host is not allowed".into());
    }

    Ok(())
}

fn is_blocked_host(host: &str) -> bool {
    let normalized = host
        .trim_matches('.')
        .trim_start_matches('[')
        .trim_end_matches(']')
        .to_ascii_lowercase();
    if normalized.is_empty()
        || normalized == "localhost"
        || normalized.ends_with(".localhost")
        || normalized == "metadata.google.internal"
    {
        return true;
    }

    if let Ok(ip) = normalized.parse::<IpAddr>() {
        return is_blocked_ip(ip);
    }

    false
}

fn is_blocked_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => is_blocked_ipv4(ip),
        IpAddr::V6(ip) => is_blocked_ipv6(ip),
    }
}

async fn validate_resolved_host(url: &Url) -> Result<(), String> {
    let Some(host) = url.host_str() else {
        return Err("url host is required".into());
    };
    if host.parse::<IpAddr>().is_ok() {
        return Ok(());
    }

    let port = url.port_or_known_default().unwrap_or(80);
    let addrs = tokio::net::lookup_host((host, port))
        .await
        .map_err(|e| format!("resolve host: {}", e))?;

    for addr in addrs {
        if is_blocked_ip(addr.ip()) {
            return Err("url host resolves to a blocked network".into());
        }
    }

    Ok(())
}

fn is_blocked_ipv4(ip: Ipv4Addr) -> bool {
    ip.is_private()
        || ip.is_loopback()
        || ip.is_link_local()
        || ip.is_broadcast()
        || ip.is_documentation()
        || ip.is_unspecified()
        || ip.is_multicast()
        || ip.octets()[0] == 0
        || ip.octets()[0] >= 224
        || ip == Ipv4Addr::new(169, 254, 169, 254)
}

fn is_blocked_ipv6(ip: Ipv6Addr) -> bool {
    ip.is_loopback()
        || ip.is_unspecified()
        || ip.is_multicast()
        || (ip.segments()[0] & 0xffc0) == 0xfe80
        || (ip.segments()[0] & 0xfe00) == 0xfc00
}

fn is_retryable_status(status: u16) -> bool {
    status == 408 || status == 409 || status == 425 || status == 429 || status >= 500
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
                let text = element_text(el);
                if !text.is_empty() {
                    return text;
                }
            }
        }
    }
    if let Ok(sel) = Selector::parse("meta[property='og:title']") {
        if let Some(el) = doc.select(&sel).next() {
            if let Some(content) = el.value().attr("content") {
                let text = content.trim().to_string();
                if !text.is_empty() {
                    return text;
                }
            }
        }
    }
    if let Ok(sel) = Selector::parse("title") {
        if let Some(el) = doc.select(&sel).next() {
            return element_text(el);
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
        "article .article-content",
        "article .read__content",
        "article .detail_text",
        ".article-content",
        ".article__content",
        ".article-body",
        ".entry-content",
        ".post-content",
        ".read__content",
        ".detail_text",
        ".story-body",
        "#article-content",
        "[itemprop='articleBody']",
    ];
    for sel_str in &selectors {
        if let Ok(sel) = Selector::parse(sel_str) {
            if let Some(el) = doc.select(&sel).next() {
                let text = element_text(el);
                if text.len() > 200 {
                    return text;
                }
            }
        }
    }
    // Fallback: article paragraphs
    if let Ok(sel) = Selector::parse("article p") {
        let paragraphs: Vec<String> = doc
            .select(&sel)
            .map(element_text)
            .filter(|s| !s.is_empty())
            .collect();
        let joined = paragraphs.join("\n\n");
        if joined.len() > 200 {
            return joined;
        }
    }
    // Final fallback: all long paragraphs
    if let Ok(sel) = Selector::parse("p") {
        let paragraphs: Vec<String> = doc
            .select(&sel)
            .map(element_text)
            .filter(|s| s.len() > 50)
            .collect();
        return paragraphs.join("\n\n");
    }
    String::new()
}

fn extract_author(doc: &Html) -> String {
    let selectors = [
        "[rel='author']",
        ".author-name",
        ".byline",
        "[itemprop='author']",
    ];
    for sel_str in &selectors {
        if let Ok(sel) = Selector::parse(sel_str) {
            if let Some(el) = doc.select(&sel).next() {
                let text = element_text(el);
                if !text.is_empty() {
                    return text;
                }
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
            if let Some(dt) = el.value().attr("datetime") {
                return dt.to_string();
            }
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
                    if !src.is_empty() {
                        return resolve_url(base_url, src);
                    }
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
                    if !k.is_empty() && seen.insert(k.clone()) {
                        tags.push(k);
                    }
                }
            }
        }
    }
    if let Ok(sel) = Selector::parse(".tags a, .post-tags a, [rel='tag']") {
        for el in doc.select(&sel) {
            let text = element_text(el);
            if !text.is_empty() && seen.insert(text.clone()) {
                tags.push(text);
            }
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

fn element_text(el: ElementRef<'_>) -> String {
    el.text()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_fetch_url_rejects_local_and_private_targets() {
        for url in [
            "http://localhost/story",
            "http://127.0.0.1/story",
            "http://10.0.0.1/story",
            "http://172.16.0.1/story",
            "http://192.168.1.1/story",
            "http://169.254.169.254/latest/meta-data",
            "http://[::1]/story",
            "file:///etc/passwd",
        ] {
            assert!(validate_fetch_url(url).is_err(), "{url} should be blocked");
        }
    }

    #[test]
    fn validate_fetch_url_accepts_public_http_targets() {
        assert!(validate_fetch_url("https://example.com/news/story").is_ok());
    }
}
