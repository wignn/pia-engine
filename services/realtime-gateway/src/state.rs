use std::sync::Arc;

use crate::config::Config;
use crate::hub::Hub;
use crate::snapshot::Snapshot;
use crate::tenant::TenantRegistry;
use crate::ticket::TicketStore;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub hub: Arc<Hub>,
    pub tenant_registry: Option<Arc<TenantRegistry>>,
    pub ticket_store: Arc<TicketStore>,
    pub snapshot: Arc<Snapshot>,
}
