pub mod server;
pub mod auth;
pub mod keys;
pub mod tenant_config;
pub mod plans;
pub mod usage;

use sqlx::PgPool;
use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Config,
    pub redis: Option<redis::Client>,
}
