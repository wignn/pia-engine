use atlsd_common::config::{get_env, get_env_u64};

#[derive(Debug, Clone)]
pub struct Config {
    pub bind_addr: String,
    pub api_keys: Vec<String>,
    pub api_key_connection_limits: std::collections::HashMap<String, i32>,
    pub log_level: String,
    pub database_url: String,
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
            api_key_connection_limits: parse_key_limits(&get_env("API_KEY_WS_CONNECTION_LIMITS", "")),
            log_level: get_env("LOG_LEVEL", "INFO"),
            database_url: get_env("DATABASE_URL", ""),
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

fn parse_key_limits(raw: &str) -> std::collections::HashMap<String, i32> {
    raw.split(',')
        .filter_map(|entry| {
            let (key, limit) = entry.split_once('=')?;
            let key = key.trim();
            let limit = limit.trim().parse::<i32>().ok()?;
            if key.is_empty() || limit < 1 {
                return None;
            }
            Some((key.to_string(), limit))
        })
        .collect()
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

#[cfg(test)]
mod tests {
    use super::parse_key_limits;

    #[test]
    fn parse_key_limits_accepts_comma_separated_key_limits() {
        let limits = parse_key_limits("primary=2, admin = 5, bad, zero=0");

        assert_eq!(limits.get("primary"), Some(&2));
        assert_eq!(limits.get("admin"), Some(&5));
        assert!(!limits.contains_key("bad"));
        assert!(!limits.contains_key("zero"));
    }
}
