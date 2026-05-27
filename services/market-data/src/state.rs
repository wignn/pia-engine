use std::sync::Arc;

use parking_lot::RwLock;
use sqlx::PgPool;
use std::collections::HashMap;

use crate::calendar::CalendarCache;
use crate::clickhouse::ClickHouseClient;
use crate::config::Config;
use crate::prices::CachedPrice;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: PgPool,
    pub clickhouse: Option<Arc<ClickHouseClient>>,
    pub prices: Arc<RwLock<HashMap<String, CachedPrice>>>,
    pub calendar: CalendarCache,
}

impl AppState {
    pub fn new(config: Config, db: PgPool, clickhouse: Option<Arc<ClickHouseClient>>) -> Self {
        Self {
            config,
            db,
            clickhouse,
            prices: Arc::new(RwLock::new(HashMap::new())),
            calendar: CalendarCache::new(),
        }
    }
}
