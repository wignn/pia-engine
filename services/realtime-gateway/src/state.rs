use std::sync::Arc;

use crate::config::Config;
use crate::hub::Hub;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub hub: Arc<Hub>,
    pub ticket_store: Arc<tokio::sync::RwLock<std::collections::HashMap<String, Ticket>>>,
}

#[derive(Clone)]
pub struct Ticket {
    pub api_key: String,
    pub expires_at: std::time::Instant,
}
