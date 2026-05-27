use std::time::Instant;

use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, USER_AGENT};
use rss::Channel;

use super::sources::NewsSource;
use super::text::{parse_rss_date, strip_html, ParsedArticle};

pub struct FeedFetchResult {
    pub status: i32,
    pub latency_ms: i64,
    pub articles: Vec<ParsedArticle>,
}

#[derive(Clone)]
pub struct RssClient {
    client: reqwest::Client,
}

impl RssClient {
    pub fn new() -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            .default_headers(default_headers())
            .timeout(std::time::Duration::from_secs(20))
            .build()?;
        Ok(Self { client })
    }

    pub fn http_client(&self) -> reqwest::Client {
        self.client.clone()
    }

    pub async fn fetch(&self, source: &NewsSource) -> anyhow::Result<FeedFetchResult> {
        let Some(url) = source.rss_url.as_deref() else {
            return Ok(FeedFetchResult {
                status: 204,
                latency_ms: 0,
                articles: Vec::new(),
            });
        };

        let started = Instant::now();
        let response = self.client.get(url).send().await?;
        let status = response.status().as_u16() as i32;
        let body = response.text().await?;
        let latency_ms = started.elapsed().as_millis().min(i64::MAX as u128) as i64;

        if !(200..300).contains(&status) {
            return Ok(FeedFetchResult {
                status,
                latency_ms,
                articles: Vec::new(),
            });
        }

        let channel = Channel::read_from(body.as_bytes())?;
        let articles = channel
            .items()
            .iter()
            .take(30)
            .filter_map(parse_item)
            .collect();

        Ok(FeedFetchResult {
            status,
            latency_ms,
            articles,
        })
    }
}

fn parse_item(item: &rss::Item) -> Option<ParsedArticle> {
    let title = item
        .title()
        .map(str::trim)
        .filter(|title| !title.is_empty())?;
    let url = item
        .link()
        .or_else(|| item.guid().map(|guid| guid.value()))
        .map(str::trim)
        .filter(|url| !url.is_empty())?;
    let summary = item
        .description()
        .map(strip_html)
        .filter(|value| !value.is_empty());
    let published_at = item.pub_date().and_then(parse_rss_date);

    Some(ParsedArticle::new(title, url, summary, published_at))
}

fn default_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("ATLSD-NewsService/1.0 (+https://wign.dev)"),
    );
    headers.insert(
        ACCEPT,
        HeaderValue::from_static("application/rss+xml, application/xml, text/xml, */*"),
    );
    headers
}
