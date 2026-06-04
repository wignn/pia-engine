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
    pub finnhub_api_key: Option<String>,
    pub finnhub_news_poll_sec: u64,
    pub finnhub_economic_calendar_poll_sec: u64,
    pub fred_api_key: Option<String>,
    pub fred_series: Vec<String>,
    pub fred_poll_sec: u64,
    pub ai_service_url: Option<String>,
}

impl Config {
    pub fn load() -> Self {
        let mut database_url = get_env(
            "DATABASE_URL",
            "postgres://postgres:***@localhost:5432/forex",
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
            finnhub_api_key: optional_env("FINNHUB_API_KEY")
                .or_else(|| optional_env("PRIMARY_FX_API_KEY")),
            finnhub_news_poll_sec: get_env_u64("FINNHUB_NEWS_POLL_SEC", 900).max(600),
            finnhub_economic_calendar_poll_sec: get_env_u64(
                "FINNHUB_ECONOMIC_CALENDAR_POLL_SEC",
                3600,
            )
            .max(1800),
            fred_api_key: optional_env("FRED_API_KEY"),
            fred_series: list_env("FRED_SERIES"),
            fred_poll_sec: get_env_u64("FRED_POLL_SEC", 21_600).max(21_600),
            ai_service_url: optional_env("AI_SERVICE_URL"),
        }
    }
}

fn optional_env(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn list_env(key: &str) -> Vec<String> {
    std::env::var(key)
        .ok()
        .map(|value| {
            value
                .split(',')
                .map(|item| item.trim().to_uppercase())
                .filter(|item| !item.is_empty())
                .collect()
        })
        .unwrap_or_default()
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
