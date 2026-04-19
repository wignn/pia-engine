use async_trait::async_trait;
use crate::error::AppError;

#[async_trait]
pub trait Scraper: Send + Sync {
    type Output: Send;
    async fn scrape(&self, url: &str) -> Result<Self::Output, AppError>;
}
