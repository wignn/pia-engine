use std::collections::HashSet;

use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct TenantContext {
    pub user_id: Uuid,
    pub api_key_id: Uuid,
    pub plan: String,
    pub is_admin: bool,
    pub requests_per_day: i32,
    pub ws_connections: i32,
    pub tv_symbols_max: i32,
    pub rate_limit_per_min: i32,
    pub can_scrape: bool,
    pub tv_symbols: HashSet<String>,
}

impl TenantContext {
    pub fn admin() -> Self {
        Self {
            user_id: Uuid::nil(),
            api_key_id: Uuid::nil(),
            plan: "enterprise".into(),
            is_admin: true,
            requests_per_day: i32::MAX,
            ws_connections: i32::MAX,
            tv_symbols_max: i32::MAX,
            rate_limit_per_min: i32::MAX,
            can_scrape: true,
            tv_symbols: HashSet::new(),
        }
    }
}
