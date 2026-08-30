use atlsd_common::config::{get_env, get_env_u64, sanitize_database_url};

#[derive(Clone, Debug)]
pub struct Config {
    pub nats_url: String,
    pub database_url: String,
    pub log_level: String,

    // Macro Batch Worker
    pub macro_enabled: bool,
    pub macro_consumer_name: String,
    pub macro_batch_size: usize,
    pub macro_ack_wait_sec: u64,
    pub macro_max_deliver: i64,

    // Usage Batch Worker
    pub usage_enabled: bool,
    pub usage_consumer_name: String,
    pub usage_batch_size: usize,
    pub usage_batch_timeout_ms: u64,
    pub usage_ack_wait_sec: u64,
    pub usage_max_deliver: i64,

    // News Enrichment Batch Worker
    pub news_enabled: bool,
    pub news_consumer_name: String,
    pub news_batch_size: usize,
    pub news_batch_timeout_ms: u64,
    pub news_ack_wait_sec: u64,
    pub news_max_deliver: i64,
}

impl Config {
    pub fn load() -> Self {
        let mut database_url = get_env(
            "DATABASE_URL",
            "postgres://postgres:postgres@localhost:5432/core",
        );
        database_url = database_url.replace("postgresql+asyncpg://", "postgres://");
        database_url = database_url.replace("postgresql://", "postgres://");
        database_url = sanitize_database_url(&database_url);

        Self {
            nats_url: get_env("NATS_URL", "nats://localhost:4222"),
            database_url,
            log_level: get_env("LOG_LEVEL", "INFO"),

            macro_enabled: get_env("MACRO_SINK_ENABLED", "true") != "false",
            macro_consumer_name: get_env("MACRO_SINK_CONSUMER", "macro-sink-postgres-v1"),
            macro_batch_size: get_env_u64("MACRO_SINK_BATCH_SIZE", 50) as usize,
            macro_ack_wait_sec: get_env_u64("MACRO_SINK_ACK_WAIT_SEC", 30).max(5),
            macro_max_deliver: get_env_u64("MACRO_SINK_MAX_DELIVER", 5).max(1) as i64,

            usage_enabled: get_env("USAGE_SINK_ENABLED", "true") != "false",
            usage_consumer_name: get_env("USAGE_SINK_CONSUMER", "usage-sink-postgres-v1"),
            usage_batch_size: get_env_u64("USAGE_SINK_BATCH_SIZE", 500) as usize,
            usage_batch_timeout_ms: get_env_u64("USAGE_SINK_BATCH_TIMEOUT_MS", 750),
            usage_ack_wait_sec: get_env_u64("USAGE_SINK_ACK_WAIT_SEC", 30).max(5),
            usage_max_deliver: get_env_u64("USAGE_SINK_MAX_DELIVER", 5).max(1) as i64,

            news_enabled: get_env("NEWS_SINK_ENABLED", "true") != "false",
            news_consumer_name: get_env("NEWS_SINK_CONSUMER", "news-sink-postgres-v1"),
            news_batch_size: get_env_u64("NEWS_SINK_BATCH_SIZE", 100) as usize,
            news_batch_timeout_ms: get_env_u64("NEWS_SINK_BATCH_TIMEOUT_MS", 1000),
            news_ack_wait_sec: get_env_u64("NEWS_SINK_ACK_WAIT_SEC", 30).max(5),
            news_max_deliver: get_env_u64("NEWS_SINK_MAX_DELIVER", 5).max(1) as i64,
        }
    }
}
