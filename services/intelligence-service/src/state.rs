use std::sync::Arc;

use sqlx::PgPool;

use crate::clickhouse::ClickHouseClient;
use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: PgPool,
    pub http: reqwest::Client,
    pub clickhouse: Option<Arc<ClickHouseClient>>,
}
