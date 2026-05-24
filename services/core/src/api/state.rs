use sqlx::PgPool;
use std::sync::Arc;

use super::usage_tracker::UsageTracker;
use crate::clickhouse::ClickHouseClient;
use crate::collector::forex::ForexCollector;
use crate::config::Config;
use crate::tenant::registry::TenantRegistry;
use crate::ws::Hub;

#[derive(Clone)]
pub struct Ticket {
    pub api_key: String,
    pub expires_at: std::time::Instant,
}

#[derive(Clone)]
pub struct AppState {
    pub hub: Arc<Hub>,
    pub db: PgPool,
    pub config: Config,
    pub forex_collector: Arc<ForexCollector>,
    pub tenant_registry: Option<Arc<TenantRegistry>>,
    pub clickhouse: Option<Arc<ClickHouseClient>>,
    pub usage_tracker: Arc<UsageTracker>,
    pub ticket_store: Arc<tokio::sync::RwLock<std::collections::HashMap<String, Ticket>>>,
}
