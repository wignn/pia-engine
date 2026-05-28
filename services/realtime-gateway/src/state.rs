use std::sync::Arc;

use crate::config::Config;
use crate::hub::Hub;
use crate::tenant::TenantRegistry;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub hub: Arc<Hub>,
    pub tenant_registry: Option<Arc<TenantRegistry>>,
    pub ticket_store: Arc<tokio::sync::RwLock<std::collections::HashMap<String, Ticket>>>,
}

#[derive(Clone)]
pub struct Ticket {
    pub api_key: String,
    pub expires_at: std::time::Instant,
}
