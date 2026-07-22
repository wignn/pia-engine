use atlsd_common::config::{get_env, get_env_u64, sanitize_database_url};

#[derive(Debug, Clone)]
pub struct Config {
    pub bind_addr: String,
    pub database_url: String,
    pub log_level: String,
    pub redis_url: String,
    pub eventbus_mode: String,
    pub nats_url: String,
    pub clickhouse_url: String,
    pub clickhouse_database: String,
    pub clickhouse_user: String,
    pub clickhouse_password: String,
    pub write_latest: bool,
    pub calendar_refresh_sec: u64,
    pub alert_notifications_enabled: bool,
    pub alert_scan_sec: u64,
    pub alert_cooldown_sec: u64,
    pub fred_api_key: String,
    pub economic_refresh_sec: u64,
    pub rates_refresh_sec: u64,
    pub eia_api_key: String,
    pub eia_sync_sec: u64,
    pub cot_sync_sec: u64,
    pub cot_data_url: String,
    pub fear_greed_sync_sec: u64,
    pub options_sync_sec: u64,
}

impl Config {
    pub fn load() -> Self {
        let mut database_url = get_env(
            "DATABASE_URL",
            "postgres://postgres:postgres@localhost:5432/forex",
        );
        database_url = database_url.replace("postgresql+asyncpg://", "postgres://");
        database_url = database_url.replace("postgresql://", "postgres://");
        database_url = sanitize_database_url(&database_url);

        let bind_addr = get_env("MARKET_DATA_BIND_ADDR", "");
        let bind_addr = if bind_addr.trim().is_empty() {
            format!("0.0.0.0:{}", get_env_u64("MARKET_DATA_PORT", 8010))
        } else {
            bind_addr
        };

        Self {
            bind_addr,
            database_url,
            log_level: get_env("LOG_LEVEL", "INFO"),
            redis_url: get_env("REDIS_URL", ""),
            eventbus_mode: get_env("EVENTBUS_MODE", "redis"),
            nats_url: get_env("NATS_URL", "nats://localhost:4222"),
            clickhouse_url: get_env("CLICKHOUSE_URL", ""),
            clickhouse_database: get_env("CLICKHOUSE_DATABASE", "market"),
            clickhouse_user: get_env("CLICKHOUSE_USER", "default"),
            clickhouse_password: get_env("CLICKHOUSE_PASSWORD", ""),
            write_latest: get_env_bool("MARKET_DATA_WRITE_LATEST", false),
            calendar_refresh_sec: get_env_u64("MARKET_CALENDAR_REFRESH_SEC", 300).max(60),
            alert_notifications_enabled: get_env_bool("ALERT_NOTIFICATIONS_ENABLED", false),
            alert_scan_sec: get_env_u64("ALERT_SCAN_SEC", 30).max(10),
            alert_cooldown_sec: get_env_u64("ALERT_COOLDOWN_SEC", 900).max(60),
            fred_api_key: get_env("FRED_API_KEY", ""),
            economic_refresh_sec: get_env_u64("ECONOMIC_REFRESH_SEC", 21600).max(600),
            rates_refresh_sec: get_env_u64("RATES_REFRESH_SEC", 21600).max(600),
            eia_api_key: get_env("EIA_API_KEY", ""),
            eia_sync_sec: get_env_u64("EIA_SYNC_SEC", 86400).max(600),
            cot_sync_sec: get_env_u64("COT_SYNC_SEC", 86400).max(600),
            cot_data_url: get_env(
                "COT_DATA_URL",
                "https://www.cftc.gov/dea/newfmt/deacot2026.txt",
            ),
            fear_greed_sync_sec: get_env_u64("FEAR_GREED_SYNC_SEC", 3600).max(600),
            options_sync_sec: get_env_u64("OPTIONS_SYNC_SEC", 3600).max(600),
        }
    }

    pub fn has_redis(&self) -> bool {
        !self.redis_url.trim().is_empty()
    }

    pub fn has_clickhouse(&self) -> bool {
        !self.clickhouse_url.trim().is_empty()
    }

    pub fn has_fred(&self) -> bool {
        !self.fred_api_key.trim().is_empty()
    }

    pub fn has_eia(&self) -> bool {
        !self.eia_api_key.trim().is_empty()
    }
}

fn get_env_bool(key: &str, default: bool) -> bool {
    match std::env::var(key) {
        Ok(value) => matches!(
            value.trim().to_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
        Err(_) => default,
    }
}
