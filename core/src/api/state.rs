use std::sync::Arc;
use sqlx::PgPool;

use crate::config::Config;
use crate::stats::StatsHub;
use crate::tenant::registry::TenantRegistry;
use crate::ws::Hub;
use super::usage_tracker::UsageTracker;

/// Shared application state accessible by all handlers.
#[derive(Clone)]
pub struct AppState {
    pub hub: Arc<Hub>,
    pub stats_hub: Arc<StatsHub>,
    pub db: PgPool,
    pub config: Config,
    pub tenant_registry: Option<Arc<TenantRegistry>>,
    pub usage_tracker: Arc<UsageTracker>,
}
