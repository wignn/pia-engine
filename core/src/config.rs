use std::env;
use url::Url;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
    pub api_keys: Vec<String>,
    pub scraper_timeout: u64,
    pub scraper_ua: String,
    pub rss_max_entries: usize,
    pub rss_fetch_sec: u64,
    pub stock_fetch_sec: u64,
    pub calendar_check_sec: u64,
    pub log_level: String,
    pub rsshub_url: String,
    pub x_usernames: String,
    pub x_poll_sec: u64,
    pub redis_url: String,
    pub redis_channel_prefix: String,
}

impl Config {
    pub fn load() -> Self {
        let mut database_url =
            get_env("DATABASE_URL", "postgres://postgres:postgres@localhost:5432/forex");
        database_url = database_url.replace("postgresql+asyncpg://", "postgres://");
        database_url = database_url.replace("postgresql://", "postgres://");
        database_url = sanitize_database_url(&database_url);

        let api_keys_raw = get_env("API_KEYS", "");
        let api_keys: Vec<String> = api_keys_raw
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Self {
            database_url,
            server_port: get_env_u64("PORT", 8000) as u16,
            api_keys,
            scraper_timeout: get_env_u64("SCRAPER_TIMEOUT", 30),
            scraper_ua: get_env(
                "SCRAPER_USER_AGENT",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
            ),
            rss_max_entries: get_env_u64("RSS_MAX_ENTRIES_PER_FEED", 50) as usize,
            rss_fetch_sec: get_env_u64("RSS_FETCH_SEC", 20),
            stock_fetch_sec: get_env_u64("STOCK_FETCH_SEC", 20),
            calendar_check_sec: get_env_u64("CALENDAR_CHECK_SEC", 60),
            log_level: get_env("LOG_LEVEL", "INFO"),

            rsshub_url: get_env("RSSHUB_URL", "http://rsshub:1200"),
            x_usernames: get_env_any(&["X_USERNAMES", "X_USERNAME"], ""),
            x_poll_sec: get_env_u64("X_POLL_SEC", 60),
            redis_url: get_env("REDIS_URL", ""),
            redis_channel_prefix: get_env("REDIS_CHANNEL_PREFIX", "world-info"),
        }
    }

    pub fn has_twitter(&self) -> bool {
        !self.x_usernames.is_empty()
    }

    pub fn has_redis(&self) -> bool {
        !self.redis_url.trim().is_empty()
    }
}

fn get_env(key: &str, fallback: &str) -> String {
    env::var(key).unwrap_or_else(|_| fallback.to_string())
}

fn get_env_u64(key: &str, fallback: u64) -> u64 {
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(fallback)
}

fn get_env_f64(key: &str, fallback: f64) -> f64 {
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(fallback)
}

fn get_env_any(keys: &[&str], fallback: &str) -> String {
    for key in keys {
        if let Ok(value) = env::var(key) {
            if !value.trim().is_empty() {
                return value;
            }
        }
    }
    fallback.to_string()
}

fn sanitize_database_url(input: &str) -> String {
    let mut url = match Url::parse(input) {
        Ok(u) => u,
        Err(_) => return input.to_string(),
    };

    let pairs: Vec<(String, String)> = url
        .query_pairs()
        .filter(|(k, _)| k != "channel_binding")
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    url.query_pairs_mut().clear();
    if !pairs.is_empty() {
        url.query_pairs_mut()
            .extend_pairs(pairs.iter().map(|(k, v)| (k.as_str(), v.as_str())));
    }

    url.to_string()
}
