use atlsd_common::config::{get_env, get_env_u64, sanitize_database_url};

#[derive(Debug, Clone)]
pub struct Config {
    pub bind_addr: String,
    pub database_url: String,
    pub redis_url: String,
    pub api_keys: Vec<String>,
    pub admin_api_key: String,
    pub log_level: String,
    pub market_data_url: String,
    pub news_service_url: String,
    pub intelligence_service_url: String,
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

        let bind_addr = get_env("API_GATEWAY_BIND_ADDR", "");
        let bind_addr = if bind_addr.trim().is_empty() {
            format!("0.0.0.0:{}", get_env_u64("API_GATEWAY_PORT", 8000))
        } else {
            bind_addr
        };

        let api_keys = get_env("API_KEYS", "")
            .split(',')
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .collect();

        Self {
            bind_addr,
            database_url,
            redis_url: get_env("REDIS_URL", ""),
            api_keys,
            admin_api_key: get_env("ADMIN_API_KEY", ""),
            log_level: get_env("LOG_LEVEL", "INFO"),
            market_data_url: get_env("MARKET_DATA_URL", "http://localhost:8010"),
            news_service_url: get_env("NEWS_SERVICE_URL", "http://localhost:8030"),
            intelligence_service_url: get_env("INTELLIGENCE_SERVICE_URL", "http://localhost:8040"),
        }
    }

    pub fn has_redis(&self) -> bool {
        !self.redis_url.trim().is_empty()
    }
}
