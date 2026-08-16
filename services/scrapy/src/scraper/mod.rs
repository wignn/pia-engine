mod extractor;
pub mod fxstreet;
pub mod http;
pub mod investing_live;

use std::time::Duration;

use crate::{error::Result, models::News, pipeline::normalize::normalize};

#[derive(Clone)]
pub struct Scraper {
    http: http::HttpClient,
}

impl Scraper {
    pub fn new(timeout: Duration, max_body_bytes: usize) -> Result<Self> {
        Ok(Self {
            http: http::HttpClient::new(timeout, max_body_bytes)?,
        })
    }

    pub async fn scrape(&self, url: &str) -> Result<News> {
        if url.contains("fxstreet.com") {
            return self.scrape_fxstreet(url).await;
        }
        if url.contains("investing.com") {
            return self.scrape_investing_live(url).await;
        }
        anyhow::bail!("unsupported source: {url}");
    }

    pub async fn scrape_fxstreet(&self, url: &str) -> Result<News> {
        let html = self.http.get_text(url).await?;
        Ok(normalize(fxstreet::FxStreetExtractor::extract(&html, url)?))
    }

    pub async fn scrape_investing_live(&self, url: &str) -> Result<News> {
        let html = self.http.get_text(url).await?;
        Ok(normalize(investing_live::InvestingLiveExtractor::extract(
            &html, url,
        )?))
    }
}
