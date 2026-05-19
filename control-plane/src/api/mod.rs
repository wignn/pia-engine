pub mod admin;
pub mod auth;
pub mod keys;
pub mod plans;
pub mod server;
pub mod tenant_config;
pub mod usage;

use crate::config::Config;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Config,
    pub redis: Option<redis::Client>,
}
