use chrono::{DateTime, Utc};
use regex::Regex;
use reqwest::Client;
use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};
use std::time::Duration;
use tracing::{error, info, warn};

static TWEET_ID_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"/status/(\d+)").unwrap());
static RE_HTML_TAGS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<[^>]+>").unwrap());

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Tweet {
    pub id: String,
    pub text: String,
    pub author_id: String,
    pub author_username: String,
    pub author_name: String,
    pub author_avatar: String,
    pub created_at: String,
    pub url: String,
    pub media_urls: Vec<String>,
}

pub struct TwitterCollector {
    rsshub_url: String,
    client: Client,
    last_seen_ids: RwLock<HashMap<String, String>>,
}

impl TwitterCollector {
    pub fn new(rsshub_url: &str, timeout: Duration) -> Self {
        Self {
            rsshub_url: rsshub_url.trim_end_matches('/').to_string(),
            client: Client::builder().timeout(timeout).build().expect("http client"),
            last_seen_ids: RwLock::new(HashMap::new()),
        }
    }

    pub async fn fetch_tweets(&self, usernames: &str) -> Vec<Tweet> {
        if self.rsshub_url.is_empty() || usernames.is_empty() {
            return vec![];
        }
        let names: Vec<&str> = usernames.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
        let mut all = Vec::new();
        for name in &names {
            let tweets = self.fetch_user_feed(name).await;
            all.extend(tweets);
        }
        info!(tweets = all.len(), users = names.len(), "twitter: fetch completed");
        all
    }

    async fn fetch_user_feed(&self, username: &str) -> Vec<Tweet> {
        let feed_url = format!("{}/twitter/user/{}", self.rsshub_url, username);
        let resp = match self.client.get(&feed_url).send().await {
            Ok(r) => r,
            Err(e) => { error!(user = username, error = %e, "twitter: rsshub fetch failed"); return vec![]; }
        };
        if !resp.status().is_success() {
            error!(user = username, status = %resp.status(), "twitter: rsshub returned error");
            return vec![];
        }
        let body = match resp.bytes().await {
            Ok(b) => b,
            Err(e) => { error!(user = username, error = %e, "twitter: read body failed"); return vec![]; }
        };

        let feed = match feed_rs::parser::parse(&body[..]) {
            Ok(f) => f,
            Err(e) => { error!(user = username, error = %e, "twitter: parse RSS failed"); return vec![]; }
        };

        let last_seen = self.last_seen_ids.read().ok()
            .and_then(|g| g.get(&username.to_lowercase()).cloned())
            .unwrap_or_default();

        let author_name = feed.title.as_ref().map(|t| {
            let name = &t.content;
            if let Some(idx) = name.find(" - ") { name[..idx].to_string() } else { name.clone() }
        }).unwrap_or_else(|| username.to_string());

        let author_avatar = feed.logo.as_ref().map(|l| l.uri.clone()).unwrap_or_default();

        let mut tweets = Vec::new();
        let mut newest_id = String::new();

        for item in &feed.entries {
            let link = item.links.first().map(|l| l.href.clone()).unwrap_or_default();
            let tweet_id = extract_tweet_id(&link)
                .or_else(|| item.id.clone().into())
                .unwrap_or_default();
            if tweet_id.is_empty() { continue; }
            if newest_id.is_empty() { newest_id = tweet_id.clone(); }
            if !last_seen.is_empty() && tweet_id == last_seen { break; }

            let description = item.summary.as_ref().map(|s| s.content.clone()).unwrap_or_default();
            let text = strip_html_tags(&description);
            let media_urls = extract_images_from_html(&description);
            let created_at = item.published
                .or(item.updated)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_default();

            tweets.push(Tweet {
                id: tweet_id,
                text,
                author_id: username.to_string(),
                author_username: username.to_string(),
                author_name: author_name.clone(),
                author_avatar: author_avatar.clone(),
                created_at,
                url: link,
                media_urls,
            });
        }

        if !newest_id.is_empty() {
            if let Ok(mut guard) = self.last_seen_ids.write() {
                guard.insert(username.to_lowercase(), newest_id);
            }
        }

        info!(user = username, count = tweets.len(), "twitter: fetched tweets");
        tweets
    }
}

fn extract_tweet_id(link: &str) -> Option<String> {
    TWEET_ID_REGEX.captures(link).map(|c| c[1].to_string())
}

fn strip_html_tags(html: &str) -> String {
    let text = RE_HTML_TAGS.replace_all(html, "");
    text.trim().to_string()
}

fn extract_images_from_html(html: &str) -> Vec<String> {
    let doc = scraper::Html::parse_fragment(html);
    let img_sel = scraper::Selector::parse("img").unwrap();
    let video_sel = scraper::Selector::parse("video").unwrap();
    let mut urls = Vec::new();
    for el in doc.select(&img_sel) {
        if let Some(src) = el.value().attr("src") {
            if !src.is_empty() { urls.push(src.to_string()); }
        }
    }
    for el in doc.select(&video_sel) {
        if let Some(poster) = el.value().attr("poster") {
            if !poster.is_empty() { urls.push(poster.to_string()); }
        }
    }
    urls
}
