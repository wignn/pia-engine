use chrono::{DateTime, Utc};
use feed_rs::parser;
use md5::{Digest, Md5};
use regex::Regex;
use reqwest::Client;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;
use std::time::Duration;
use tokio::sync::Semaphore;
use tracing::{error, info};

use super::rss::FeedSource;

/// A stock news entry parsed from RSS feeds.
#[derive(Debug, Clone)]
pub struct StockNewsEntry {
    pub title: String,
    pub link: String,
    pub content: String,
    pub published_at: Option<DateTime<Utc>>,
    pub author: String,
    pub tags: Vec<String>,
    pub content_hash: String,
    pub source_name: String,
    pub category: String,
    pub tickers: Vec<String>,
}

/// Indonesian stock market RSS feed sources.
pub fn indonesia_stock_feeds() -> Vec<FeedSource> {
    vec![
        FeedSource {
            name: "CNBC Indonesia - Market".into(),
            url: "https://www.cnbcindonesia.com/market".into(),
            rss_url: "https://www.cnbcindonesia.com/market/rss".into(),
            category: "market".into(),
        },
        FeedSource {
            name: "Investing.com Indonesia - Market".into(),
            url: "https://id.investing.com".into(),
            rss_url: "https://id.investing.com/rss/news_25.rss".into(),
            category: "market".into(),
        },
        FeedSource {
            name: "Tempo.co - Market".into(),
            url: "https://www.tempo.co".into(),
            rss_url: "https://rss.tempo.co/bisnis".into(),
            category: "market".into(),
        },
        FeedSource {
            name: "Detik - Market".into(),
            url: "https://finance.detik.com".into(),
            rss_url: "https://finance.detik.com/rss".into(),
            category: "market".into(),
        },
        FeedSource {
            name: "CNN - Market".into(),
            url: "https://www.cnnindonesia.com/ekonomi".into(),
            rss_url: "https://www.cnnindonesia.com/ekonomi/rss".into(),
            category: "market".into(),
        },
    ]
}

static STOCK_KEYWORDS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    [
        "ihsg", "idx", "bei", "bursa efek", "saham", "emiten", "dividen",
        "ipo", "right issue", "stock split", "buyback", "tender offer",
        "listing", "delisting", "suspensi", "trading halt",
        "naik", "turun", "melemah", "menguat", "bullish", "bearish",
        "koreksi", "rally", "rebound", "profit taking", "window dressing",
        "laba", "rugi", "pendapatan", "omzet", "revenue", "net profit",
        "laporan keuangan", "kuartal", "semester", "tahunan",
        "eps", "per", "pbv", "roe", "roa", "der",
        "akuisisi", "merger", "divestasi", "spin off", "rights issue",
        "obligasi", "sukuk", "private placement",
        "perbankan", "bank", "properti", "konstruksi", "tambang", "mining",
        "energi", "telekomunikasi", "consumer", "fmcg", "farmasi",
        "otomotif", "infrastruktur", "bumn",
        "bbca", "bbri", "bmri", "bbni", "tlkm", "asii", "unvr", "hmsp",
        "ggrm", "icbp", "indf", "klbf", "pgas", "ptba", "adro", "antm",
        "inco", "mdka", "goto", "buka", "arto", "bris",
    ]
    .into_iter()
    .collect()
});

static KNOWN_TICKERS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    [
        "BBCA", "BBRI", "BMRI", "BBNI", "TLKM", "ASII", "UNVR", "HMSP",
        "GGRM", "ICBP", "INDF", "KLBF", "PGAS", "PTBA", "ADRO", "ANTM",
        "INCO", "MDKA", "GOTO", "BUKA", "ARTO", "BRIS", "BBTN", "SMGR",
        "INTP", "EXCL", "ISAT", "TOWR", "TBIG", "MNCN", "SCMA", "AKRA",
        "UNTR", "MEDC", "ESSA", "ACES", "MAPI", "ERAA", "SIDO", "KAEF",
        "CPIN", "JPFA", "MAIN", "SRIL", "TKIM", "INKP", "BRPT", "TPIA",
        "AMRT", "MIDI", "LPPF", "MYOR", "ROTI", "ULTJ", "MLBI", "DLTA",
        "IHSG", "JKSE",
    ]
    .into_iter()
    .collect()
});

static TICKER_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\b([A-Z]{4})\b").unwrap());
static RE_HTML: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<[^>]+>").unwrap());

/// Extract known IDX tickers from text.
fn extract_tickers(text: &str) -> Vec<String> {
    let upper = text.to_uppercase();
    let mut seen = HashSet::new();
    let mut valid = Vec::new();

    for cap in TICKER_PATTERN.captures_iter(&upper) {
        let m = &cap[1];
        if KNOWN_TICKERS.contains(m) && seen.insert(m.to_string()) {
            valid.push(m.to_string());
        }
    }

    valid
}

/// Check if a stock news entry is relevant based on tickers or keywords.
fn is_relevant_stock(entry: &StockNewsEntry) -> bool {
    if !entry.tickers.is_empty() {
        return true;
    }

    let text = format!("{} {}", entry.title, entry.content).to_lowercase();
    let words: HashSet<&str> = text.split_whitespace().collect();

    for kw in STOCK_KEYWORDS.iter() {
        if !kw.contains(' ') {
            if words.contains(kw) {
                return true;
            }
        } else if text.contains(kw) {
            return true;
        }
    }

    false
}

/// Collector for Indonesian stock market news from multiple RSS feeds.
pub struct StockCollector {
    client: Client,
    semaphore: Semaphore,
    timeout: Duration,
    user_agent: String,
}

impl StockCollector {
    pub fn new(user_agent: &str, timeout: Duration) -> Self {
        Self {
            client: Client::builder()
                .timeout(timeout)
                .user_agent(user_agent)
                .pool_max_idle_per_host(3)
                .build()
                .expect("failed to build HTTP client"),
            semaphore: Semaphore::new(6),
            timeout,
            user_agent: user_agent.to_string(),
        }
    }

    /// Fetch latest stock news entries, deduplicated and sorted by date.
    pub async fn fetch_latest(&self, max_entries: usize) -> Vec<StockNewsEntry> {
        let results = self.fetch_all_feeds().await;

        let mut all: Vec<StockNewsEntry> = results.into_values().flatten().collect();

        all.sort_by(|a, b| {
            let a_time = a.published_at.unwrap_or(DateTime::UNIX_EPOCH);
            let b_time = b.published_at.unwrap_or(DateTime::UNIX_EPOCH);
            b_time.cmp(&a_time)
        });

        let mut seen = HashSet::new();
        let unique: Vec<StockNewsEntry> = all
            .into_iter()
            .filter(|e| seen.insert(e.content_hash.clone()))
            .take(max_entries)
            .collect();

        unique
    }

    async fn fetch_all_feeds(&self) -> HashMap<String, Vec<StockNewsEntry>> {
        let feeds = indonesia_stock_feeds();
        let mut handles = Vec::new();

        for feed in feeds {
            let feed = feed.clone();
            handles.push(async move {
                let entries = self.fetch_feed(&feed).await;
                tokio::time::sleep(Duration::from_millis(100)).await;
                (feed.name.clone(), entries)
            });
        }

        let results: Vec<_> = futures_util::future::join_all(handles).await;
        results.into_iter().collect()
    }

    async fn fetch_feed(&self, source: &FeedSource) -> Vec<StockNewsEntry> {
        let _permit = match self.semaphore.acquire().await {
            Ok(p) => p,
            Err(_) => return vec![],
        };

        let resp = match self
            .client
            .get(&source.rss_url)
            .header("User-Agent", &self.user_agent)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                error!(source = %source.name, error = %e, "stock feed fetch failed");
                return vec![];
            }
        };

        let body = match resp.bytes().await {
            Ok(b) => b,
            Err(e) => {
                error!(source = %source.name, error = %e, "read body failed");
                return vec![];
            }
        };

        let feed = match parser::parse(&body[..]) {
            Ok(f) => f,
            Err(e) => {
                error!(source = %source.name, error = %e, "parse stock feed failed");
                return vec![];
            }
        };

        let limit = 20.min(feed.entries.len());
        let mut entries = Vec::new();

        for item in feed.entries.iter().take(limit) {
            if let Some(entry) = self.parse_item(item, source) {
                if is_relevant_stock(&entry) {
                    entries.push(entry);
                }
            }
        }

        info!(source = %source.name, entries = entries.len(), "stock feed fetched");
        entries
    }

    fn parse_item(&self, item: &feed_rs::model::Entry, source: &FeedSource) -> Option<StockNewsEntry> {
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

        let content = RE_HTML.replace_all(&content, "").trim().to_string();
        let content = if content.len() > 2000 {
            content[..2000].to_string()
        } else {
            content
        };

        let published_at = item.published.or(item.updated);

        let tags: Vec<String> = item
            .categories
            .iter()
            .map(|c| c.term.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let hash_content = format!("{}{}", title, link);
        let hash = format!("{:x}", Md5::digest(hash_content.as_bytes()));

        let tickers = extract_tickers(&format!("{} {}", title, content));

        let author = item
            .authors
            .first()
            .map(|a| a.name.clone())
            .unwrap_or_default();

        Some(StockNewsEntry {
            title,
            link,
            content,
            published_at,
            author,
            tags,
            content_hash: hash,
            source_name: source.name.clone(),
            category: source.category.clone(),
            tickers,
        })
    }
}
