use std::env;

#[derive(Debug, Clone)]
pub struct MarketSymbolConfig {
    pub provider_symbol: String,
    pub public_symbol: String,
    pub asset_type: String,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub primary_fx_api_key: String,
    pub secondary_fx_api_key: String,
    pub primary_fx_ws_url: String,
    pub crypto_feed_ws_url: String,
    pub tradingview_quote_url_template: String,
    pub primary_fx_symbols: Vec<MarketSymbolConfig>,
    pub secondary_fx_symbols: Vec<MarketSymbolConfig>,
    pub index_feed_symbols: Vec<MarketSymbolConfig>,
    pub stock_feed_symbols: Vec<MarketSymbolConfig>,
    pub crypto_symbols: Vec<String>,
    pub crypto_feed_enabled: bool,
    pub redis_url: String,
    pub redis_channel_prefix: String,
    pub eventbus_mode: String,
    pub nats_url: String,
    pub reconnect_base_sec: u64,
    pub reconnect_max_sec: u64,
    pub market_check_interval_sec: u64,
    pub health_bind_addr: String,
    pub health_stale_after_sec: u64,
    pub log_level: String,
}

impl Config {
    pub fn load() -> Self {
        let crypto_symbols_raw = get_env("CRYPTO_SYMBOLS", "BTCUSDT,ETHUSDT,SOLUSDT,BNBUSDT");
        let crypto_symbols: Vec<String> = crypto_symbols_raw
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_lowercase())
            .collect();

        let primary_fx_symbols = parse_symbol_mappings(&get_env("PRIMARY_FX_SYMBOLS", ""), "forex");
        let secondary_fx_symbols =
            parse_symbol_mappings(&get_env("SECONDARY_FX_SYMBOLS", ""), "forex");

        Self {
            primary_fx_api_key: get_env("PRIMARY_FX_API_KEY", ""),
            secondary_fx_api_key: get_env(
                "SECONDARY_FX_API_KEY",
                &get_env("SECONDRY_FX_API_KEY", ""),
            ),
            primary_fx_ws_url: get_env("PRIMARY_FX_WS_URL", ""),
            crypto_feed_ws_url: get_env("CRYPTO_FEED_WS_URL", ""),
            tradingview_quote_url_template: get_env("TRADINGVIEW_QUOTE_URL_TEMPLATE", ""),
            primary_fx_symbols,
            secondary_fx_symbols,
            index_feed_symbols: parse_symbol_mappings(&get_env("INDEX_FEED_SYMBOLS", ""), "index"),
            stock_feed_symbols: parse_symbol_mappings(&get_env("STOCK_FEED_SYMBOLS", ""), "stock"),
            crypto_symbols,
            crypto_feed_enabled: get_env("CRYPTO_FEED_ENABLED", "true")
                .to_lowercase()
                .eq("true"),
            redis_url: get_env("REDIS_URL", ""),
            redis_channel_prefix: get_env("REDIS_CHANNEL_PREFIX", "ingestion"),
            eventbus_mode: get_env("EVENTBUS_MODE", "redis"),
            nats_url: get_env("NATS_URL", "nats://localhost:4222"),
            reconnect_base_sec: get_env_u64("RECONNECT_BASE_SEC", 5),
            reconnect_max_sec: get_env_u64("RECONNECT_MAX_SEC", 300),
            market_check_interval_sec: get_env_u64("MARKET_CHECK_INTERVAL_SEC", 30),
            health_bind_addr: get_env("INGESTION_HEALTH_BIND_ADDR", "0.0.0.0:8091"),
            health_stale_after_sec: get_env_u64("INGESTION_HEALTH_STALE_AFTER_SEC", 180),
            log_level: get_env("LOG_LEVEL", "INFO"),
        }
    }

    pub fn has_primary_fx(&self) -> bool {
        !self.primary_fx_api_key.trim().is_empty() && !self.primary_fx_symbols.is_empty()
    }

    pub fn has_secondary_fx(&self) -> bool {
        !self.secondary_fx_api_key.trim().is_empty() && !self.secondary_fx_symbols.is_empty()
    }

    pub fn has_redis(&self) -> bool {
        !self.redis_url.trim().is_empty()
    }
}

fn parse_symbol_mappings(raw: &str, default_asset_type: &str) -> Vec<MarketSymbolConfig> {
    raw.split(',')
        .filter_map(|item| {
            let mut parts = item.split('|').map(str::trim);
            let provider_symbol = parts.next()?.to_string();
            let public_symbol = parts.next()?.to_uppercase();
            let asset_type = parts.next().unwrap_or(default_asset_type).to_lowercase();

            if provider_symbol.is_empty() || public_symbol.is_empty() {
                return None;
            }

            Some(MarketSymbolConfig {
                provider_symbol,
                public_symbol,
                asset_type,
            })
        })
        .collect()
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
