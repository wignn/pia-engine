use atlsd_common::config::{get_env, get_env_u64, sanitize_database_url};

#[derive(Debug, Clone)]
pub struct Config {
    pub bind_addr: String,
    pub database_url: String,
    pub log_level: String,
    pub run_pipelines: bool,
    pub eventbus_mode: String,
    pub nats_url: String,
    pub realtime_poll_sec: u64,
    pub rss_fetch_sec: u64,
    pub stock_fetch_sec: u64,
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

        let bind_addr = get_env("NEWS_SERVICE_BIND_ADDR", "");
        let bind_addr = if bind_addr.trim().is_empty() {
            format!("0.0.0.0:{}", get_env_u64("NEWS_SERVICE_PORT", 8030))
        } else {
            bind_addr
        };

        Self {
            bind_addr,
            database_url,
            log_level: get_env("LOG_LEVEL", "INFO"),
            run_pipelines: env_bool("NEWS_SERVICE_RUN_PIPELINES", false),
            eventbus_mode: get_env("EVENTBUS_MODE", "redis"),
            nats_url: get_env("NATS_URL", "nats://localhost:4222"),
            realtime_poll_sec: get_env_u64("NEWS_REALTIME_POLL_SEC", 10).max(1),
            rss_fetch_sec: get_env_u64("RSS_FETCH_SEC", 60).max(15),
            stock_fetch_sec: get_env_u64("STOCK_FETCH_SEC", 300).max(60),
        }
    }
}

fn env_bool(key: &str, default: bool) -> bool {
    match std::env::var(key) {
        Ok(value) => matches!(
            value.trim().to_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
        Err(_) => default,
    }
}
