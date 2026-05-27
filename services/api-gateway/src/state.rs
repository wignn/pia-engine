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
}
