use std::sync::Arc;
use std::time::Instant;

use crate::config::Config;
use crate::tenant::TenantRegistry;
use crate::usage::UsageTracker;

#[derive(Clone)]
pub struct CachedPriceSnapshot {
    pub bytes: axum::body::Bytes,
    pub cached_at: Instant,
}

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub http: reqwest::Client,
    pub nats: Option<async_nats::Client>,
    pub tenant_registry: Arc<TenantRegistry>,
    pub usage_tracker: Arc<UsageTracker>,
    pub internal_api_key: Option<String>,
    pub price_cache: Arc<parking_lot::RwLock<Option<CachedPriceSnapshot>>>,
}
