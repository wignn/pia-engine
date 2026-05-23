use atlsd_auth::api_key::hash_key;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;

use super::context::TenantContext;

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
    tv_symbols_max: i32,
    rate_limit_per_min: i32,
    expires_at: Option<DateTime<Utc>>,
}

/// Cached per-user config.
#[derive(Debug, Clone, Default)]
struct UserConfig {
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

    /// Load all active keys and configs from DB.
    pub async fn reload(&self) {
        // Load keys with plan info
        let rows: Result<Vec<TenantRow>, _> = sqlx::query_as(
            "SELECT k.key_hash, k.user_id, k.id, u.plan, k.is_active, u.is_active, \
                    COALESCE(p.can_scrape, FALSE), \
                    COALESCE(p.requests_per_day, 100), \
                    COALESCE(p.ws_connections, 1), \
                    COALESCE(p.tv_symbols_max, 3), \
                    COALESCE(p.rate_limit_per_min, 10), \
                    k.expires_at \
             FROM api_keys k \
             JOIN users u ON u.id = k.user_id \
             LEFT JOIN plans p ON p.id = u.plan",
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
                            tv_symbols_max: tv_max,
                            rate_limit_per_min: rlm,
                            expires_at: expires,
                        },
                    );
                }
                let count = map.len();
                *self.keys.write().await = map;
                info!(keys = count, "tenant registry: keys reloaded");
            }
            Err(e) => error!(error = %e, "tenant registry: failed to load keys"),
        }

        // Load tenant configs
        let cfg_rows: Result<Vec<(Uuid, String, serde_json::Value)>, _> =
            sqlx::query_as("SELECT user_id, config_key, config_value FROM tenant_configs")
                .fetch_all(&self.db)
                .await;

        match cfg_rows {
            Ok(rows) => {
                let mut map: HashMap<Uuid, UserConfig> = HashMap::new();
                for (uid, key, val) in rows {
                    let entry = map.entry(uid).or_default();
                    if key.as_str() == "tv_symbols" {
                        if let Some(arr) = val.as_array() {
                            for item in arr {
                                if let Some(s) = item.as_str() {
                                    entry.tv_symbols.insert(s.to_string());
                                }
                            }
                        }
                    }
                }
                let count = map.len();
                *self.configs.write().await = map;
                info!(users = count, "tenant registry: configs reloaded");
            }
            Err(e) => error!(error = %e, "tenant registry: failed to load configs"),
        }
    }

    pub async fn validate_key(&self, raw_key: &str) -> Option<TenantContext> {
        let hash = hash_key(raw_key);
        let cached = {
            let keys = self.keys.read().await;
            keys.get(&hash).cloned()
        };

        let cached = match cached {
            Some(cached) => cached,
            None => self.load_key_from_db(&hash).await?,
        };

        self.context_from_cached_key(cached).await
    }

    async fn context_from_cached_key(&self, cached: CachedKey) -> Option<TenantContext> {
        if !cached.is_active || !cached.user_active {
            return None;
        }

        if let Some(expires) = cached.expires_at {
            if Utc::now() > expires {
                warn!(key_id = %cached.key_id, "API key expired");
                return None;
            }
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
            tv_symbols_max: cached.tv_symbols_max,
            rate_limit_per_min: cached.rate_limit_per_min,
            can_scrape: cached.can_scrape,
            tv_symbols: user_cfg.tv_symbols,
        })
    }

    async fn load_key_from_db(&self, hash: &str) -> Option<CachedKey> {
        let row: Result<Option<TenantRow>, _> = sqlx::query_as(
            "SELECT k.key_hash, k.user_id, k.id, u.plan, k.is_active, u.is_active, \
                    COALESCE(p.can_scrape, FALSE), \
                    COALESCE(p.requests_per_day, 100), \
                    COALESCE(p.ws_connections, 1), \
                    COALESCE(p.tv_symbols_max, 3), \
                    COALESCE(p.rate_limit_per_min, 10), \
                    k.expires_at \
             FROM api_keys k \
             JOIN users u ON u.id = k.user_id \
             LEFT JOIN plans p ON p.id = u.plan \
             WHERE k.key_hash = $1",
        )
        .bind(hash)
        .fetch_optional(&self.db)
        .await;

        let (hash, uid, kid, plan, kactive, uactive, scrape, rpd, wsc, tv_max, rlm, expires) = row
            .map_err(|e| {
                error!(error = %e, "tenant registry: failed to load key on cache miss");
                e
            })
            .ok()??;

        self.load_config_for_user(uid).await;

        let cached = CachedKey {
            user_id: uid,
            key_id: kid,
            plan,
            is_active: kactive,
            user_active: uactive,
            can_scrape: scrape,
            requests_per_day: rpd,
            ws_connections: wsc,
            tv_symbols_max: tv_max,
            rate_limit_per_min: rlm,
            expires_at: expires,
        };

        self.keys.write().await.insert(hash, cached.clone());
        info!(user_id = %uid, key_id = %kid, "tenant registry: key loaded on cache miss");
        Some(cached)
    }

    async fn load_user_from_db(&self, user_id: Uuid) {
        let rows: Result<Vec<TenantRow>, _> = sqlx::query_as(
            "SELECT k.key_hash, k.user_id, k.id, u.plan, k.is_active, u.is_active, \
                    COALESCE(p.can_scrape, FALSE), \
                    COALESCE(p.requests_per_day, 100), \
                    COALESCE(p.ws_connections, 1), \
                    COALESCE(p.tv_symbols_max, 3), \
                    COALESCE(p.rate_limit_per_min, 10), \
                    k.expires_at \
             FROM api_keys k \
             JOIN users u ON u.id = k.user_id \
             LEFT JOIN plans p ON p.id = u.plan \
             WHERE k.user_id = $1",
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await;

        match rows {
            Ok(rows) => {
                let mut keys = self.keys.write().await;
                keys.retain(|_, key| key.user_id != user_id);
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
                    tv_max,
                    rlm,
                    expires,
                ) in rows
                {
                    keys.insert(
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
                            tv_symbols_max: tv_max,
                            rate_limit_per_min: rlm,
                            expires_at: expires,
                        },
                    );
                }
            }
            Err(e) => {
                error!(user_id = %user_id, error = %e, "tenant registry: failed to load user keys")
            }
        }

        self.load_config_for_user(user_id).await;
    }

    async fn load_config_for_user(&self, user_id: Uuid) {
        let cfg_rows: Result<Vec<(String, serde_json::Value)>, _> = sqlx::query_as(
            "SELECT config_key, config_value FROM tenant_configs WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await;

        match cfg_rows {
            Ok(rows) => {
                let mut user_cfg = UserConfig::default();
                for (key, val) in rows {
                    if key.as_str() == "tv_symbols" {
                        if let Some(arr) = val.as_array() {
                            for item in arr {
                                if let Some(s) = item.as_str() {
                                    user_cfg.tv_symbols.insert(s.to_string());
                                }
                            }
                        }
                    }
                }
                self.configs.write().await.insert(user_id, user_cfg);
            }
            Err(e) => {
                error!(user_id = %user_id, error = %e, "tenant registry: failed to load user config")
            }
        }
    }

    /// Background task: listen for Redis config_changed events + periodic fallback reload.
    pub async fn run_sync(
        self: Arc<Self>,
        redis_client: Option<redis::Client>,
        channel_prefix: String,
        mut shutdown: tokio::sync::watch::Receiver<bool>,
    ) {
        if let Some(client) = redis_client {
            let registry = self.clone();
            let prefix = channel_prefix.clone();
            let mut shutdown_redis = shutdown.clone();

            tokio::spawn(async move {
                loop {
                    if let Err(e) =
                        Self::redis_subscribe_loop(&registry, &client, &prefix, &mut shutdown_redis)
                            .await
                    {
                        warn!(error = %e, "tenant config subscriber error, reconnecting in 5s");
                    }

                    tokio::select! {
                        _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {}
                        _ = shutdown_redis.changed() => { break; }
                    }
                }
            });

            info!("tenant registry: Redis config subscriber started");
        }

        // Keep periodic fallback reload (60s) for consistency.
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            tokio::select! {
                _ = interval.tick() => { self.reload().await; }
                _ = shutdown.changed() => { break; }
            }
        }
    }

    /// Subscribe to the `{prefix}:tenant:config_changed` Redis channel and
    /// reload the in-memory cache every time the control-plane publishes an event.
    async fn redis_subscribe_loop(
        registry: &Arc<Self>,
        client: &redis::Client,
        prefix: &str,
        shutdown: &mut tokio::sync::watch::Receiver<bool>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use futures_util::StreamExt;

        let mut pubsub = client.get_async_pubsub().await?;
        let channel = format!("{}:tenant:config_changed", prefix);
        pubsub.subscribe(&channel).await?;
        info!(channel = %channel, "tenant config subscriber connected");

        let mut msg_stream = pubsub.on_message();

        loop {
            tokio::select! {
                msg = msg_stream.next() => {
                    match msg {
                        Some(msg) => {
                            let payload: redis::RedisResult<String> = msg.get_payload();
                            match payload
                                .ok()
                                .and_then(|payload| serde_json::from_str::<serde_json::Value>(&payload).ok())
                                .and_then(|payload| payload.get("user_id").and_then(|id| id.as_str()).map(str::to_string))
                                .and_then(|id| Uuid::parse_str(&id).ok())
                            {
                                Some(user_id) => {
                                    info!(user_id = %user_id, "tenant config changed event received, reloading user cache");
                                    registry.load_user_from_db(user_id).await;
                                }
                                None => {
                                    info!("tenant config changed event received, reloading cache");
                                    registry.reload().await;
                                }
                            }
                        }
                        None => {
                            warn!("tenant config subscriber stream ended");
                            return Ok(());
                        }
                    }
                }
                _ = shutdown.changed() => { break; }
            }
        }
        Ok(())
    }
}
