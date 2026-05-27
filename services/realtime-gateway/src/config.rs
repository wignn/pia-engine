use atlsd_common::config::{get_env, get_env_u64};

#[derive(Debug, Clone)]
pub struct Config {
    pub bind_addr: String,
    pub api_keys: Vec<String>,
    pub log_level: String,
    pub redis_url: String,
    pub redis_channel_prefix: String,
    pub redis_subscribe_enabled: bool,
    pub eventbus_mode: String,
    pub nats_url: String,
}

impl Config {
    pub fn load() -> Self {
        let bind_addr = get_env("REALTIME_GATEWAY_BIND_ADDR", "");
        let bind_addr = if bind_addr.trim().is_empty() {
            format!("0.0.0.0:{}", get_env_u64("REALTIME_GATEWAY_PORT", 8020))
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
            api_keys,
            log_level: get_env("LOG_LEVEL", "INFO"),
            redis_url: get_env("REDIS_URL", ""),
            redis_channel_prefix: get_env("REDIS_CHANNEL_PREFIX", "world-info"),
            redis_subscribe_enabled: env_bool("REALTIME_REDIS_SUBSCRIBE_ENABLED", true),
            eventbus_mode: get_env("EVENTBUS_MODE", "redis"),
            nats_url: get_env("NATS_URL", "nats://localhost:4222"),
        }
    }

    pub fn has_redis(&self) -> bool {
        self.redis_subscribe_enabled && !self.redis_url.trim().is_empty()
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
