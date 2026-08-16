use std::sync::Arc;

use crate::config::Config;
use crate::tenant::TenantRegistry;
use crate::usage::UsageTracker;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub http: reqwest::Client,
    pub tenant_registry: Arc<TenantRegistry>,
    pub usage_tracker: Arc<UsageTracker>,
    /// Shared secret forwarded to internal services (INTERNAL_API_KEY env).
    pub internal_api_key: Option<String>,
}
