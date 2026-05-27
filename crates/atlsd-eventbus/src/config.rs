#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventBusMode {
    Redis,
    Nats,
    Dual,
    Noop,
}

impl EventBusMode {
    pub fn from_env_value(value: &str) -> Self {
        match value.trim().to_lowercase().as_str() {
            "nats" => Self::Nats,
            "dual" => Self::Dual,
            "noop" => Self::Noop,
            _ => Self::Redis,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EventBusConfig {
    pub mode: EventBusMode,
    pub nats_url: String,
    pub redis_prefix: String,
}

impl EventBusConfig {
    pub fn from_env(redis_prefix_default: impl Into<String>) -> Self {
        Self {
            mode: EventBusMode::from_env_value(
                &std::env::var("EVENTBUS_MODE").unwrap_or_else(|_| "redis".to_string()),
            ),
            nats_url: std::env::var("NATS_URL")
                .unwrap_or_else(|_| "nats://localhost:4222".to_string()),
            redis_prefix: std::env::var("EVENTBUS_REDIS_PREFIX")
                .unwrap_or_else(|_| redis_prefix_default.into()),
        }
    }
}
