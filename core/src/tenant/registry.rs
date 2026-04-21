use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;

use super::context::TenantContext;

/// Cached API key → tenant info mapping.
#[derive(Debug, Clone)]
struct CachedKey {
    user_id: Uuid,
    key_id: Uuid,
    plan: String,
    is_active: bool,
    user_active: bool,
    can_scrape: bool,
    requests_per_day: i32,
    ws_connections: i32,
    rate_limit_per_min: i32,
}

/// Cached per-user config.
#[derive(Debug, Clone, Default)]
struct UserConfig {
    tv_symbols: HashSet<String>,
}

/// In-memory registry of API keys and tenant configs, synced from DB.
pub struct TenantRegistry {
    keys: Arc<RwLock<HashMap<String, CachedKey>>>,    // key_hash → CachedKey
    configs: Arc<RwLock<HashMap<Uuid, UserConfig>>>,   // user_id → UserConfig
    db: PgPool,
}

impl TenantRegistry {
    pub fn new(db: PgPool) -> Arc<Self> {
        Arc::new(Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
            configs: Arc::new(RwLock::new(HashMap::new())),
            db,
        })
    }

    /// Load all active keys and configs from DB.
    pub async fn reload(&self) {
        // Load keys with plan info
        let rows: Result<Vec<(String, Uuid, Uuid, String, bool, bool, bool, i32, i32, i32)>, _> = sqlx::query_as(
            "SELECT k.key_hash, k.user_id, k.id, u.plan, k.is_active, u.is_active, \
                    COALESCE(p.can_scrape, FALSE), \
                    COALESCE(p.requests_per_day, 100), \
                    COALESCE(p.ws_connections, 1), \
                    COALESCE(p.rate_limit_per_min, 10) \
             FROM api_keys k \
             JOIN users u ON u.id = k.user_id \
             LEFT JOIN plans p ON p.id = u.plan"
        )
        .fetch_all(&self.db)
        .await;

        match rows {
            Ok(rows) => {
                let mut map = HashMap::new();
                for (hash, uid, kid, plan, kactive, uactive, scrape, rpd, wsc, rlm) in rows {
                    map.insert(hash, CachedKey {
                        user_id: uid, key_id: kid, plan, is_active: kactive,
                        user_active: uactive, can_scrape: scrape,
                        requests_per_day: rpd, ws_connections: wsc, rate_limit_per_min: rlm,
                    });
                }
                let count = map.len();
                *self.keys.write().await = map;
                info!(keys = count, "tenant registry: keys reloaded");
            }
            Err(e) => error!(error = %e, "tenant registry: failed to load keys"),
        }

        // Load tenant configs
        let cfg_rows: Result<Vec<(Uuid, String, serde_json::Value)>, _> = sqlx::query_as(
            "SELECT user_id, config_key, config_value FROM tenant_configs"
        )
        .fetch_all(&self.db)
        .await;

        match cfg_rows {
            Ok(rows) => {
                let mut map: HashMap<Uuid, UserConfig> = HashMap::new();
                for (uid, key, val) in rows {
                    let entry = map.entry(uid).or_default();
                    match key.as_str() {
                        "tv_symbols" => {
                            if let Some(arr) = val.as_array() {
                                for item in arr {
                                    if let Some(s) = item.as_str() {
                                        entry.tv_symbols.insert(s.to_string());
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                let count = map.len();
                *self.configs.write().await = map;
                info!(users = count, "tenant registry: configs reloaded");
            }
            Err(e) => error!(error = %e, "tenant registry: failed to load configs"),
        }
    }

    /// Validate an API key and return TenantContext.
    pub async fn validate_key(&self, raw_key: &str) -> Option<TenantContext> {
        let hash = hash_key(raw_key);
        let keys = self.keys.read().await;
        let cached = keys.get(&hash)?;

        if !cached.is_active || !cached.user_active {
            return None;
        }

        let configs = self.configs.read().await;
        let user_cfg = configs.get(&cached.user_id).cloned().unwrap_or_default();

        Some(TenantContext {
            user_id: cached.user_id,
            api_key_id: cached.key_id,
            plan: cached.plan.clone(),
            is_admin: false,
            requests_per_day: cached.requests_per_day,
            ws_connections: cached.ws_connections,
            rate_limit_per_min: cached.rate_limit_per_min,
            can_scrape: cached.can_scrape,
            tv_symbols: user_cfg.tv_symbols,
        })
    }

    /// Background task: periodically reload registry.
    pub async fn run_sync(self: Arc<Self>, mut shutdown: tokio::sync::watch::Receiver<bool>) {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            tokio::select! {
                _ = interval.tick() => { self.reload().await; }
                _ = shutdown.changed() => { break; }
            }
        }
    }
}

fn hash_key(raw: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    hex::encode(hasher.finalize())
}
