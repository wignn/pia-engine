use chrono::{DateTime, Utc};
use feed_rs::parser;
use reqwest::header::{ACCEPT, ACCEPT_LANGUAGE, CACHE_CONTROL, ETAG, IF_MODIFIED_SINCE, IF_NONE_MATCH, LAST_MODIFIED, REFERER};
use reqwest::Client;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::{RwLock, Semaphore};
use tracing::{error, info, warn};

/// A single RSS feed entry after parsing.
#[derive(Debug, Clone)]
pub struct RSSEntry {
    pub title: String,
    pub link: String,
    pub content: String,
    pub published_at: Option<DateTime<Utc>>,
    pub author: String,
    pub tags: Vec<String>,
    pub content_hash: String,
    pub source_name: String,
}

/// A configured RSS feed source.
#[derive(Debug, Clone)]
pub struct FeedSource {
    pub name: String,
    pub url: String,
    pub rss_url: String,
    pub category: String,
}

/// Compute a content hash from URL and title.
pub fn compute_content_hash(url: &str, title: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("{}|{}", url, title));
    format!("{:x}", hasher.finalize())
}

/// Default forex news RSS feed sources.
pub fn default_forex_feeds() -> Vec<FeedSource> {
    vec![
        FeedSource {
            name: "Thomson Reuters".into(),
            url: "https://ir.thomsonreuters.com".into(),
            rss_url: "https://ir.thomsonreuters.com/rss/news-releases.xml?items=15".into(),
            category: "general".into(),
        },
        FeedSource {
            name: "InvestingLive".into(),
            url: "https://investinglive.com".into(),
            rss_url: "https://investinglive.com/feed/news/".into(),
            category: "forex".into(),
        },
        FeedSource {
            name: "FXStreet".into(),
            url: "https://www.fxstreet-id.com".into(),
            rss_url: "https://www.fxstreet-id.com/rss/news".into(),
            category: "forex".into(),
        },
        FeedSource {
            name: "Investing.com - Forex News".into(),
            url: "https://id.investing.com/news/forex-news".into(),
            rss_url: "https://id.investing.com/rss/news_301.rss".into(),
            category: "forex".into(),
        },
        FeedSource {
            name: "Investing.com - Economic Indicators".into(),
            url: "https://id.investing.com/news/economic-indicators".into(),
            rss_url: "https://id.investing.com/rss/news_95.rss".into(),
            category: "economic".into(),
        },
        FeedSource {
            name: "Federal Reserve".into(),
            url: "https://www.federalreserve.gov".into(),
            rss_url: "https://www.federalreserve.gov/feeds/press_all.xml".into(),
            category: "central_bank".into(),
        },
        FeedSource {
            name: "ECB".into(),
            url: "https://www.ecb.europa.eu".into(),
            rss_url: "https://www.ecb.europa.eu/rss/press.html".into(),
            category: "central_bank".into(),
        },
    ]
}

/// Look up a feed name by its RSS URL.
pub fn feed_name_by_url(rss_url: &str) -> String {
    for f in default_forex_feeds() {
        if f.rss_url == rss_url {
            return f.name;
        }
    }

    let lower = rss_url.to_lowercase();
    if lower.contains("fxstreet") {
        "FXStreet".into()
    } else if lower.contains("investing.com") {
        "Investing.com".into()
    } else if lower.contains("reuters") {
        "Reuters".into()
    } else if lower.contains("federalreserve") {
        "Federal Reserve".into()
    } else if lower.contains("ecb.europa") {
        "ECB".into()
    } else {
        "Unknown".into()
    }
}

/// Concurrent RSS feed collector with semaphore-limited parallelism.
pub struct RSSCollector {
    client: Client,
    max_entries: usize,
    semaphore: Semaphore,
    user_agent: String,
    source_state: RwLock<HashMap<String, SourceFetchState>>,
}

#[derive(Debug, Clone, Default)]
struct SourceFetchState {
    etag: Option<String>,
    last_modified: Option<String>,
    last_success_at: Option<DateTime<Utc>>,
    last_error_at: Option<DateTime<Utc>>,
    blocked_until: Option<DateTime<Utc>>,
    consecutive_403: u32,
    success_count: u64,
    error_count: u64,
    forbidden_count: u64,
    parse_error_count: u64,
    last_status: Option<u16>,
    last_latency_ms: Option<u128>,
    next_allowed_poll_at: Option<DateTime<Utc>>,
}

enum AttemptResult {
    Parsed(Vec<RSSEntry>, Option<String>, Option<String>, u128),
    NotModified(Option<String>, Option<String>, u128),
    Forbidden,
    RetryableHttp(u16),
    NonRetryableHttp(u16),
    Transport(String),
    Body(String),
    Parse(String, String),
}

impl RSSCollector {
    pub fn new(max_entries: usize, user_agent: &str, timeout: Duration) -> Self {
        Self {
            client: Client::builder()
                .timeout(timeout)
                .user_agent(user_agent)
                .pool_max_idle_per_host(5)
                .build()
                .expect("failed to build HTTP client"),
            max_entries,
            semaphore: Semaphore::new(6),
            user_agent: user_agent.to_string(),
            source_state: RwLock::new(HashMap::new()),
        }
    }

    /// Fetch a single RSS feed and parse its entries.
    pub async fn fetch_feed(&self, source: &FeedSource) -> Vec<RSSEntry> {
        let _permit = match self.semaphore.acquire().await {
            Ok(p) => p,
            Err(_) => return vec![],
        };

        if self.is_source_blocked(source).await {
            warn!(source = %source.name, "feed temporarily paused by circuit breaker");
            return vec![];
        }

        if !self.is_source_due(source).await {
            return vec![];
        }

        tokio::time::sleep(Duration::from_millis(compute_jitter_ms(source))).await;

        let mut last_error: Option<String> = None;
        for url in feed_candidate_urls(source) {
            for attempt in 1..=3 {
                match self.fetch_attempt(source, &url).await {
                    AttemptResult::Parsed(entries, etag, last_modified, latency_ms) => {
                        self.on_source_success(source, etag, last_modified, Some(200), latency_ms)
                            .await;
                        self.log_source_metrics(source).await;
                        info!(source = %source.name, url = %url, entries = entries.len(), attempt, "feed fetched");
                        return entries;
                    }
                    AttemptResult::NotModified(etag, last_modified, latency_ms) => {
                        self.on_source_success(source, etag, last_modified, Some(304), latency_ms)
                            .await;
                        self.log_source_metrics(source).await;
                        info!(source = %source.name, url = %url, attempt, "feed not modified");
                        return vec![];
                    }
                    AttemptResult::Forbidden => {
                        self.on_source_forbidden(source).await;
                        self.log_source_metrics(source).await;
                        error!(source = %source.name, url = %url, "feed returned non-success status: 403 Forbidden");
                        break;
                    }
                    AttemptResult::RetryableHttp(status) => {
                        last_error = Some(format!("retryable http status {status}"));
                        warn!(source = %source.name, url = %url, attempt, status, "retryable feed status");
                        tokio::time::sleep(Duration::from_millis(250 * attempt as u64)).await;
                        continue;
                    }
                    AttemptResult::NonRetryableHttp(status) => {
                        last_error = Some(format!("http status {status}"));
                        self.on_source_error(source, Some(status), false).await;
                        error!(source = %source.name, url = %url, status, "feed returned non-success status");
                        break;
                    }
                    AttemptResult::Transport(err) => {
                        last_error = Some(format!("transport error: {err}"));
                        warn!(source = %source.name, url = %url, attempt, error = %err, "feed request failed, retrying");
                        tokio::time::sleep(Duration::from_millis(250 * attempt as u64)).await;
                    }
                    AttemptResult::Body(err) => {
                        last_error = Some(format!("body read error: {err}"));
                        warn!(source = %source.name, url = %url, attempt, error = %err, "feed body read failed, retrying");
                        tokio::time::sleep(Duration::from_millis(250 * attempt as u64)).await;
                    }
                    AttemptResult::Parse(err, body_head) => {
                        last_error = Some(format!("parse error: {err}"));
                        self.on_source_error(source, Some(200), true).await;
                        warn!(source = %source.name, url = %url, attempt, error = %err, body_head = %body_head, "parse feed failed");
                        tokio::time::sleep(Duration::from_millis(300 * attempt as u64)).await;
                    }
                }
            }

            warn!(source = %source.name, url = %url, "switching to fallback feed url");
        }

        self.on_source_error(source, None, false).await;
        self.log_source_metrics(source).await;
        error!(source = %source.name, error = ?last_error, "feed fetch exhausted retries");
        vec![]
    }

    /// Fetch all feeds concurrently and return results keyed by RSS URL.
    pub async fn fetch_all_feeds(
        &self,
        feeds: &[FeedSource],
    ) -> std::collections::HashMap<String, Vec<RSSEntry>> {
        let mut handles = Vec::new();

        for feed in feeds {
            let feed = feed.clone();
            let this = &self;
            handles.push(async move {
                let entries = this.fetch_feed(&feed).await;
                tokio::time::sleep(Duration::from_millis(100)).await;
                (feed.rss_url.clone(), entries)
            });
        }

        let results: Vec<_> = futures_util::future::join_all(handles).await;
        results.into_iter().collect()
    }

    fn parse_item(&self, item: &feed_rs::model::Entry, source_name: &str) -> Option<RSSEntry> {
        let title = item.title.as_ref()?.content.trim().to_string();
        let link = item.links.first()?.href.trim().to_string();

        if title.is_empty() || link.is_empty() {
            return None;
        }

        let content = item
            .content
            .as_ref()
            .and_then(|c| c.body.clone())
            .or_else(|| item.summary.as_ref().map(|s| s.content.clone()))
            .unwrap_or_default();

        let published_at = item.published.or(item.updated);

        let author = item
            .authors
            .first()
            .map(|a| a.name.clone())
            .unwrap_or_default();

        let tags: Vec<String> = item
            .categories
            .iter()
            .map(|c| c.term.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let hash = compute_content_hash(&link, &title);

        Some(RSSEntry {
            title,
            link,
            content,
            published_at,
            author,
            tags,
            content_hash: hash,
            source_name: source_name.to_string(),
        })
    }

    async fn fetch_attempt(&self, source: &FeedSource, url: &str) -> AttemptResult {
        let (etag, last_modified) = self.get_conditional_headers(source).await;

        let mut req = self
            .client
            .get(url)
            .header("User-Agent", &self.user_agent)
            .header(ACCEPT, "application/rss+xml, application/atom+xml, application/xml;q=0.9, text/xml;q=0.8, */*;q=0.1")
            .header(ACCEPT_LANGUAGE, "en-US,en;q=0.9,id;q=0.8")
            .header(CACHE_CONTROL, "no-cache")
            .header(REFERER, &source.url);

        if let Some(value) = etag {
            req = req.header(IF_NONE_MATCH, value);
        }
        if let Some(value) = last_modified {
            req = req.header(IF_MODIFIED_SINCE, value);
        }

        let started = std::time::Instant::now();
        let resp = match req.send().await {
            Ok(r) => r,
            Err(e) => return AttemptResult::Transport(e.to_string()),
        };

        let latency_ms = started.elapsed().as_millis();
        let status = resp.status();

        let etag_next = resp
            .headers()
            .get(ETAG)
            .and_then(|v| v.to_str().ok())
            .map(|v| v.to_string());
        let last_modified_next = resp
            .headers()
            .get(LAST_MODIFIED)
            .and_then(|v| v.to_str().ok())
            .map(|v| v.to_string());

        if status.as_u16() == 304 {
            return AttemptResult::NotModified(etag_next, last_modified_next, latency_ms);
        }

        if status.as_u16() == 403 {
            return AttemptResult::Forbidden;
        }

        if status.as_u16() == 429 || status.is_server_error() {
            return AttemptResult::RetryableHttp(status.as_u16());
        }

        if !status.is_success() {
            return AttemptResult::NonRetryableHttp(status.as_u16());
        }

        let body = match resp.bytes().await {
            Ok(b) => b,
            Err(e) => return AttemptResult::Body(e.to_string()),
        };

        let normalized = normalize_xml_body(&body);
        let feed = match parser::parse(&normalized[..]) {
            Ok(f) => f,
            Err(e) => {
                let head = String::from_utf8_lossy(&normalized[..normalized.len().min(160)]).replace('\n', " ");
                return AttemptResult::Parse(e.to_string(), head);
            }
        };

        let limit = self.max_entries.min(feed.entries.len());
        let mut entries = Vec::with_capacity(limit);
        for item in feed.entries.iter().take(limit) {
            if let Some(entry) = self.parse_item(item, &source.name) {
                entries.push(entry);
            }
        }

        AttemptResult::Parsed(entries, etag_next, last_modified_next, latency_ms)
    }

    async fn get_conditional_headers(&self, source: &FeedSource) -> (Option<String>, Option<String>) {
        let state = self.source_state.read().await;
        if let Some(s) = state.get(&source.name) {
            (s.etag.clone(), s.last_modified.clone())
        } else {
            (None, None)
        }
    }

    async fn is_source_due(&self, source: &FeedSource) -> bool {
        let state = self.source_state.read().await;
        let Some(s) = state.get(&source.name) else {
            return true;
        };
        match s.next_allowed_poll_at {
            Some(ts) => Utc::now() >= ts,
            None => true,
        }
    }

    async fn is_source_blocked(&self, source: &FeedSource) -> bool {
        let state = self.source_state.read().await;
        let Some(s) = state.get(&source.name) else {
            return false;
        };
        match s.blocked_until {
            Some(ts) => Utc::now() < ts,
            None => false,
        }
    }

    async fn on_source_success(
        &self,
        source: &FeedSource,
        etag: Option<String>,
        last_modified: Option<String>,
        status: Option<u16>,
        latency_ms: u128,
    ) {
        let mut state = self.source_state.write().await;
        let s = state.entry(source.name.clone()).or_default();
        s.success_count += 1;
        s.consecutive_403 = 0;
        s.last_success_at = Some(Utc::now());
        s.last_status = status;
        s.last_latency_ms = Some(latency_ms);
        s.blocked_until = None;
        s.next_allowed_poll_at = Some(Utc::now() + chrono::Duration::seconds(per_source_poll_sec(source) as i64));
        if etag.is_some() {
            s.etag = etag;
        }
        if last_modified.is_some() {
            s.last_modified = last_modified;
        }
    }

    async fn on_source_forbidden(&self, source: &FeedSource) {
        let mut state = self.source_state.write().await;
        let s = state.entry(source.name.clone()).or_default();
        s.error_count += 1;
        s.forbidden_count += 1;
        s.consecutive_403 += 1;
        s.last_error_at = Some(Utc::now());
        s.last_status = Some(403);

        let cooldown_min = if s.consecutive_403 >= 5 {
            30
        } else if s.consecutive_403 >= 3 {
            10
        } else {
            2
        };
        s.blocked_until = Some(Utc::now() + chrono::Duration::minutes(cooldown_min));
        s.next_allowed_poll_at = s.blocked_until;
    }

    async fn on_source_error(&self, source: &FeedSource, status: Option<u16>, is_parse_error: bool) {
        let mut state = self.source_state.write().await;
        let s = state.entry(source.name.clone()).or_default();
        s.error_count += 1;
        if is_parse_error {
            s.parse_error_count += 1;
        }
        s.last_error_at = Some(Utc::now());
        s.last_status = status;
        s.next_allowed_poll_at = Some(Utc::now() + chrono::Duration::seconds(per_source_poll_sec(source) as i64));
    }

    async fn log_source_metrics(&self, source: &FeedSource) {
        let state = self.source_state.read().await;
        let Some(s) = state.get(&source.name) else {
            return;
        };

        let total = s.success_count + s.error_count;
        let success_rate = if total == 0 {
            1.0
        } else {
            s.success_count as f64 / total as f64
        };

        info!(
            source = %source.name,
            success = s.success_count,
            errors = s.error_count,
            forbidden = s.forbidden_count,
            parse_errors = s.parse_error_count,
            consecutive_403 = s.consecutive_403,
            success_rate,
            last_status = ?s.last_status,
            last_latency_ms = ?s.last_latency_ms,
            last_success_at = ?s.last_success_at,
            "feed source metrics"
        );
    }
}

fn normalize_xml_body(body: &[u8]) -> Vec<u8> {
    let mut bytes = body;

    // Strip UTF-8 BOM if present.
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        bytes = &bytes[3..];
    }

    // Some upstream/CDN responses may prepend noise before the XML root.
    let mut start = 0usize;
    for needle in [b"<?xml".as_slice(), b"<rss".as_slice(), b"<feed".as_slice()] {
        if let Some(pos) = find_subslice(bytes, needle) {
            start = if start == 0 { pos } else { start.min(pos) };
        }
    }

    bytes[start..].to_vec()
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack.windows(needle.len()).position(|w| w == needle)
}

fn feed_candidate_urls(source: &FeedSource) -> Vec<String> {
    if source.name.eq_ignore_ascii_case("Thomson Reuters") {
        vec![
            source.rss_url.clone(),
            "https://ir.thomsonreuters.com/rss/news-releases.xml".to_string(),
        ]
    } else {
        vec![source.rss_url.clone()]
    }
}

fn per_source_poll_sec(source: &FeedSource) -> u64 {
    if source.name.eq_ignore_ascii_case("Thomson Reuters") {
        120
    } else {
        45
    }
}

fn compute_jitter_ms(source: &FeedSource) -> u64 {
    let now_nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let name_hash = source
        .name
        .bytes()
        .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
    80 + ((now_nanos ^ name_hash) % 700)
}
