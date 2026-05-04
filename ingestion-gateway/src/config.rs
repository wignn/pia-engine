use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    /// Finnhub WebSocket API key
    pub finnhub_api_key: String,
    /// Tiingo WebSocket API key
    pub tiingo_api_key: String,
    /// Binance trade stream symbols (lowercase, e.g. "btcusdt")
    pub binance_symbols: Vec<String>,
    /// Whether Binance worker is enabled (default: true)
    pub binance_enabled: bool,
    /// Redis connection URL
    pub redis_url: String,
    /// Redis channel prefix for published messages (default: "ingestion")
    pub redis_channel_prefix: String,
    /// Base reconnect delay in seconds (default: 5)
    pub reconnect_base_sec: u64,
    /// Maximum reconnect delay in seconds (default: 300)
    pub reconnect_max_sec: u64,
    /// How often to check market hours while connected, in seconds (default: 30)
    pub market_check_interval_sec: u64,
    /// Log level (default: INFO)
    pub log_level: String,
}

impl Config {
    pub fn load() -> Self {
        let binance_symbols_raw = get_env(
            "BINANCE_SYMBOLS",
            "btcusdt,ethusdt,solusdt,bnbusdt,xrpusdt,dogeusdt,adausdt",
        );
        let binance_symbols: Vec<String> = binance_symbols_raw
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();

        Self {
            finnhub_api_key: get_env("FINNHUB_API_KEY", ""),
            tiingo_api_key: get_env("TIINGO_API_KEY", ""),
            binance_symbols,
            binance_enabled: get_env("BINANCE_ENABLED", "true")
                .to_lowercase()
                .eq("true"),
            redis_url: get_env("REDIS_URL", ""),
            redis_channel_prefix: get_env("REDIS_CHANNEL_PREFIX", "ingestion"),
            reconnect_base_sec: get_env_u64("RECONNECT_BASE_SEC", 5),
            reconnect_max_sec: get_env_u64("RECONNECT_MAX_SEC", 300),
            market_check_interval_sec: get_env_u64("MARKET_CHECK_INTERVAL_SEC", 30),
            log_level: get_env("LOG_LEVEL", "INFO"),
        }
    }

    pub fn has_finnhub(&self) -> bool {
        !self.finnhub_api_key.trim().is_empty()
    }

    pub fn has_tiingo(&self) -> bool {
        !self.tiingo_api_key.trim().is_empty()
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
