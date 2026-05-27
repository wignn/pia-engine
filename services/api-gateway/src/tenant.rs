use atlsd_auth::api_key::hash_key;
use atlsd_domain::tenant::TenantContext;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;

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
    x_usernames_max: i32,
    tv_symbols_max: i32,
    rate_limit_per_min: i32,
    expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default)]
struct UserConfig {
    x_usernames: HashSet<String>,
    tv_symbols: HashSet<String>,
}

pub struct TenantRegistry {
    keys: Arc<RwLock<HashMap<String, CachedKey>>>,
    configs: Arc<RwLock<HashMap<Uuid, UserConfig>>>,
    db: PgPool,
}

type TenantRow = (
    String,
    Uuid,
    Uuid,
    String,
    bool,
    bool,
    bool,
    i32,
    i32,
    i32,
    i32,
    i32,
    Option<DateTime<Utc>>,
);

impl TenantRegistry {
    pub fn new(db: PgPool) -> Arc<Self> {
        Arc::new(Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
            configs: Arc::new(RwLock::new(HashMap::new())),
            db,
        })
    }

    pub async fn reload(&self) {
        let rows: Result<Vec<TenantRow>, _> = sqlx::query_as(
            "SELECT k.key_hash, k.user_id, k.id, u.plan, k.is_active, u.is_active, COALESCE(p.can_scrape, FALSE), COALESCE(p.requests_per_day, 100), COALESCE(p.ws_connections, 1), COALESCE(p.x_usernames_max, 1), COALESCE(p.tv_symbols_max, 3), COALESCE(p.rate_limit_per_min, 10), k.expires_at FROM api_keys k JOIN users u ON u.id = k.user_id LEFT JOIN plans p ON p.id = u.plan",
        )
        .fetch_all(&self.db)
        .await;

        match rows {
            Ok(rows) => {
                let mut map = HashMap::new();
                for (
                    hash,
                    uid,
                    kid,
                    plan,
                    kactive,
                    uactive,
                    scrape,
                    rpd,
                    wsc,
                    x_max,
                    tv_max,
                    rlm,
                    expires,
                ) in rows
                {
                    map.insert(
                        hash,
                        CachedKey {
                            user_id: uid,
                            key_id: kid,
                            plan,
                            is_active: kactive,
                            user_active: uactive,
                            can_scrape: scrape,
                            requests_per_day: rpd,
                            ws_connections: wsc,
                            x_usernames_max: x_max,
                            tv_symbols_max: tv_max,
                            rate_limit_per_min: rlm,
                            expires_at: expires,
                        },
                    );
                }
                let count = map.len();
                *self.keys.write().await = map;
                info!(keys = count, "api-gateway tenant keys reloaded");
            }
            Err(err) => error!(error = %err, "api-gateway failed to load tenant keys"),
        }

        let cfg_rows: Result<Vec<(Uuid, String, serde_json::Value)>, _> =
            sqlx::query_as("SELECT user_id, config_key, config_value FROM tenant_configs")
                .fetch_all(&self.db)
                .await;
        if let Ok(rows) = cfg_rows {
            let mut map: HashMap<Uuid, UserConfig> = HashMap::new();
            for (uid, key, val) in rows {
                let entry = map.entry(uid).or_default();
                if let Some(arr) = val.as_array() {
                    for item in arr {
                        if let Some(s) = item.as_str() {
                            match key.as_str() {
                                "tv_symbols" => {
                                    entry.tv_symbols.insert(s.to_uppercase());
                                }
                                "x_usernames" => {
                                    entry
                                        .x_usernames
                                        .insert(s.trim_start_matches('@').to_lowercase());
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            *self.configs.write().await = map;
        }
    }

    pub async fn validate_key(&self, raw_key: &str) -> Option<TenantContext> {
        let hash = hash_key(raw_key);
        let cached = self.keys.read().await.get(&hash).cloned()?;
        if !cached.is_active
            || !cached.user_active
            || cached
                .expires_at
                .is_some_and(|expires| Utc::now() > expires)
        {
            warn!(key_id = %cached.key_id, "api-gateway rejected inactive/expired API key");
            return None;
        }
        let config = self
            .configs
            .read()
            .await
            .get(&cached.user_id)
            .cloned()
            .unwrap_or_default();
        Some(TenantContext {
            user_id: cached.user_id,
            api_key_id: cached.key_id,
            plan: cached.plan,
            is_admin: false,
            requests_per_day: cached.requests_per_day,
            ws_connections: cached.ws_connections,
            x_usernames_max: cached.x_usernames_max,
            tv_symbols_max: cached.tv_symbols_max,
            rate_limit_per_min: cached.rate_limit_per_min,
            can_scrape: cached.can_scrape,
            x_usernames: config.x_usernames,
            tv_symbols: config.tv_symbols,
        })
    }
}
