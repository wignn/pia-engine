use crate::error::AppError;
use async_trait::async_trait;

#[async_trait]
#[allow(dead_code)]
pub trait Scraper: Send + Sync {
    type Output: Send;
    async fn scrape(&self, url: &str) -> Result<Self::Output, AppError>;
}
