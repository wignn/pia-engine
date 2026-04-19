use async_trait::async_trait;

use crate::error::AppError;

/// Trait for extensible data collectors.
///
/// Implement this trait to add new scraping sources to the platform.
/// Each collector fetches data from an external source and returns
/// a vector of typed items.
#[async_trait]
pub trait Collector: Send + Sync {
    type Item: Send;

    /// Perform the collection and return fetched items.
    async fn collect(&self) -> Result<Vec<Self::Item>, AppError>;

    /// Human-readable name of this collector.
    fn name(&self) -> &str;
}
