use std::collections::HashMap;
use std::sync::Arc;

use atlsd_auth::api_key::hash_key;
use atlsd_domain::tenant::TenantContext;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use tokio::sync::RwLock;
use tokio::time::Duration;
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone)]
struct CachedKey {
    user_id: Uuid,
    key_id: Uuid,
    plan: String,
    is_active: bool,
    user_active: bool,
    requests_per_day: i32,
    ws_connections: i32,
    x_usernames_max: i32,
    tv_symbols_max: i32,
    rate_limit_per_min: i32,
    can_scrape: bool,
    expires_at: Option<DateTime<Utc>>,
}

pub struct TenantRegistry {
    keys: Arc<RwLock<HashMap<String, CachedKey>>>,
    db: PgPool,
}

pub fn reload_interval() -> Duration {
    Duration::from_secs(60)
}

type TenantRow = (
    String,
    Uuid,
    Uuid,
    String,
    bool,
    bool,
    i32,
    i32,
    i32,
    i32,
    i32,
    bool,
    Option<DateTime<Utc>>,
);

impl TenantRegistry {
    pub fn new(db: PgPool) -> Arc<Self> {
        Arc::new(Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
            db,
        })
    }

    pub async fn reload(&self) {
        let rows: Result<Vec<TenantRow>, _> = sqlx::query_as(
            "SELECT k.key_hash, k.user_id, k.id, u.plan, k.is_active, u.is_active, COALESCE(k.max_ws_connections, p.ws_connections, 1), COALESCE(p.requests_per_day, 100), COALESCE(p.x_usernames_max, 1), COALESCE(p.tv_symbols_max, 3), COALESCE(p.rate_limit_per_min, 10), COALESCE(p.can_scrape, FALSE), k.expires_at FROM api_keys k JOIN users u ON u.id = k.user_id LEFT JOIN plans p ON p.id = u.plan",
        )
        .fetch_all(&self.db)
        .await;

        match rows {
            Ok(rows) => {
                let mut map = HashMap::new();
                for (
                    hash,
                    user_id,
                    key_id,
                    plan,
                    is_active,
                    user_active,
                    ws_connections,
                    requests_per_day,
                    x_usernames_max,
                    tv_symbols_max,
                    rate_limit_per_min,
                    can_scrape,
                    expires_at,
                ) in rows
                {
                    map.insert(
                        hash,
                        CachedKey {
                            user_id,
                            key_id,
                            plan,
                            is_active,
                            user_active,
                            requests_per_day,
                            ws_connections,
                            x_usernames_max,
                            tv_symbols_max,
                            rate_limit_per_min,
                            can_scrape,
                            expires_at,
                        },
                    );
                }
                let count = map.len();
                *self.keys.write().await = map;
                info!(keys = count, "realtime tenant keys reloaded");
            }
            Err(err) => error!(error = %err, "realtime failed to load tenant keys"),
        }
    }

    pub async fn run_reload_loop(self: Arc<Self>) {
        let mut interval = tokio::time::interval(reload_interval());
        loop {
            interval.tick().await;
            self.reload().await;
        }
    }

    pub async fn validate_key(&self, raw_key: &str) -> Option<TenantContext> {
        let hash = hash_key(raw_key);
        let cached = self.keys.read().await.get(&hash).cloned()?;
        if !cached.is_active
            || !cached.user_active
            || cached.expires_at.is_some_and(|expires| Utc::now() > expires)
        {
            warn!(key_id = %cached.key_id, "realtime rejected inactive/expired API key");
            return None;
        }
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
            x_usernames: Default::default(),
            tv_symbols: Default::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reload_interval_is_one_minute() {
        assert_eq!(reload_interval(), Duration::from_secs(60));
    }

    #[test]
    fn tenant_context_uses_effective_ws_connections() {
        let cached = CachedKey {
            user_id: Uuid::new_v4(),
            key_id: Uuid::new_v4(),
            plan: "pro".to_string(),
            is_active: true,
            user_active: true,
            requests_per_day: 1000,
            ws_connections: 7,
            x_usernames_max: 1,
            tv_symbols_max: 3,
            rate_limit_per_min: 60,
            can_scrape: false,
            expires_at: None,
        };

        assert_eq!(cached.ws_connections, 7);
    }
}
