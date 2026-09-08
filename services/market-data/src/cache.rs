use moka::future::Cache;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, Mutex};

use crate::clickhouse::HistoryPage;

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct HistoryCacheKey {
    pub symbol: String,
    pub resolution: String,
    pub limit: usize,
    pub before: Option<i64>,
}

#[derive(Clone)]
pub struct MarketHistoryCache {
    live_cache: Cache<HistoryCacheKey, Arc<HistoryPage>>,
    historical_cache: Cache<HistoryCacheKey, Arc<HistoryPage>>,
    in_flight: Arc<Mutex<HashMap<HistoryCacheKey, broadcast::Sender<Arc<HistoryPage>>>>>,
}

impl MarketHistoryCache {
    pub fn new() -> Self {
        Self {
            live_cache: Cache::builder()
                .time_to_live(Duration::from_millis(2500))
                .max_capacity(2000)
                .build(),
            historical_cache: Cache::builder()
                .time_to_live(Duration::from_secs(120))
                .max_capacity(5000)
                .build(),
            in_flight: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_or_fetch<F, Fut>(
        &self,
        key: HistoryCacheKey,
        fetch: F,
    ) -> anyhow::Result<Arc<HistoryPage>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = anyhow::Result<HistoryPage>>,
    {
        let is_live = key.before.is_none();
        let target_cache = if is_live {
            &self.live_cache
        } else {
            &self.historical_cache
        };

        if let Some(cached) = target_cache.get(&key).await {
            return Ok(cached);
        }

        let mut rx = {
            let mut in_flight = self.in_flight.lock().await;
            if let Some(sender) = in_flight.get(&key) {
                sender.subscribe()
            } else {
                let (tx, _rx) = broadcast::channel(1);
                in_flight.insert(key.clone(), tx);
                drop(in_flight);

                let result = fetch().await;
                let mut in_flight = self.in_flight.lock().await;
                let sender = in_flight.remove(&key);

                match result {
                    Ok(data) => {
                        let arc_data = Arc::new(data);
                        target_cache.insert(key, arc_data.clone()).await;
                        if let Some(tx) = sender {
                            let _ = tx.send(arc_data.clone());
                        }
                        return Ok(arc_data);
                    }
                    Err(err) => {
                        return Err(err);
                    }
                }
            }
        };

        match rx.recv().await {
            Ok(data) => Ok(data),
            Err(_) => Err(anyhow::anyhow!("Coalesced request cancelled")),
        }
    }
}
