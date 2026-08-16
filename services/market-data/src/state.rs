use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use sqlx::PgPool;
use tokio::sync::mpsc;

use crate::calendar::CalendarCache;
use crate::candle_engine::CandleEngineHandle;
use crate::clickhouse::ClickHouseClient;
use crate::config::Config;
use crate::prices::CachedPrice;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: PgPool,
    pub clickhouse: Option<Arc<ClickHouseClient>>,
    pub tick_tx: Option<mpsc::Sender<(CachedPrice, DateTime<Utc>)>>,
    pub candle: Option<Arc<CandleEngineHandle>>,
    pub metrics: Arc<atlsd_observability::MetricsRegistry>,
    pub prices: Arc<RwLock<HashMap<String, CachedPrice>>>,
    pub calendar: CalendarCache,
}

impl AppState {
    pub fn new(
        config: Config,
        db: PgPool,
        clickhouse: Option<Arc<ClickHouseClient>>,
        tick_tx: Option<mpsc::Sender<(CachedPrice, DateTime<Utc>)>>,
        candle: Option<Arc<CandleEngineHandle>>,
        metrics: Arc<atlsd_observability::MetricsRegistry>,
    ) -> Self {
        Self {
            config,
            db,
            clickhouse,
            tick_tx,
            candle,
            metrics,
            prices: Arc::new(RwLock::new(HashMap::new())),
            calendar: CalendarCache::new(),
        }
    }
}
