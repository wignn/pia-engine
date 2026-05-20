use crate::error::AppError;
use async_trait::async_trait;

/// Trait for extensible data processing pipelines.
#[async_trait]
#[allow(dead_code)]
pub trait Pipeline: Send + Sync {
    async fn run(&self) -> Result<(), AppError>;
    fn name(&self) -> &str;
}
