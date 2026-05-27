use atlsd_common::config::{get_env, get_env_u64, sanitize_database_url};

#[derive(Debug, Clone)]
pub struct Config {
    pub bind_addr: String,
    pub database_url: String,
    pub log_level: String,
    pub ai_service_url: String,
    pub clickhouse_url: String,
    pub clickhouse_database: String,
    pub clickhouse_user: String,
    pub clickhouse_password: String,
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

        let bind_addr = get_env("INTELLIGENCE_SERVICE_BIND_ADDR", "");
        let bind_addr = if bind_addr.trim().is_empty() {
            format!("0.0.0.0:{}", get_env_u64("INTELLIGENCE_SERVICE_PORT", 8040))
        } else {
            bind_addr
        };

        Self {
            bind_addr,
            database_url,
            log_level: get_env("LOG_LEVEL", "INFO"),
            ai_service_url: get_env("AI_SERVICE_URL", "http://localhost:5000"),
            clickhouse_url: get_env("CLICKHOUSE_URL", ""),
            clickhouse_database: get_env("CLICKHOUSE_DATABASE", "market"),
            clickhouse_user: get_env("CLICKHOUSE_USER", "default"),
            clickhouse_password: get_env("CLICKHOUSE_PASSWORD", ""),
        }
    }

    pub fn has_clickhouse(&self) -> bool {
        !self.clickhouse_url.trim().is_empty()
    }
}
