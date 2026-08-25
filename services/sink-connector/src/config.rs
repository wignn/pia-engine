use atlsd_common::config::{get_env, get_env_u64, sanitize_database_url};

#[derive(Clone, Debug)]
pub struct Config {
    pub nats_url: String,
    pub database_url: String,
    pub consumer_name: String,
    pub ack_wait_sec: u64,
    pub max_deliver: i64,
    pub log_level: String,
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

        Self {
            nats_url: get_env("NATS_URL", "nats://localhost:4222"),
            database_url,
            consumer_name: get_env("MACRO_SINK_CONSUMER", "macro-sink-postgres-v1"),
            ack_wait_sec: get_env_u64("MACRO_SINK_ACK_WAIT_SEC", 30).max(5),
            max_deliver: get_env_u64("MACRO_SINK_MAX_DELIVER", 5).max(1) as i64,
            log_level: get_env("LOG_LEVEL", "INFO"),
        }
    }
}
