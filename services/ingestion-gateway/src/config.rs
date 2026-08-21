use std::collections::HashSet;
use std::env;
use tracing::{info, warn};

pub const MAX_NON_CRYPTO_SYMBOLS: usize = 100;
pub const MAX_SYMBOLS_PER_API_KEY: usize = 50;

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
    pub options_sync_sec: u64,
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

        let primary_fx_symbols = parse_symbol_mappings(
            &get_env("PRIMARY_FX_SYMBOLS", ""),
            "forex",
            MAX_SYMBOLS_PER_API_KEY,
        );
        let secondary_fx_symbols = parse_symbol_mappings(
            &get_env("SECONDARY_FX_SYMBOLS", ""),
            "forex",
            MAX_SYMBOLS_PER_API_KEY,
        );
        let secondary_fx_symbols = if secondary_fx_symbols.is_empty() {
            let legacy = get_env("SECONDARY_SYMBOLS", "");
            if !legacy.trim().is_empty() {
                warn!("SECONDARY_SYMBOLS is deprecated; use SECONDARY_FX_SYMBOLS with provider|public|asset_type mappings");
            }
            parse_symbol_mappings(&legacy, "forex", MAX_SYMBOLS_PER_API_KEY)
        } else {
            secondary_fx_symbols
        };

        if primary_fx_symbols.len() == MAX_SYMBOLS_PER_API_KEY {
            warn!(
                max = MAX_SYMBOLS_PER_API_KEY,
                "primary API-key symbol limit reached"
            );
        }
        if secondary_fx_symbols.len() == MAX_SYMBOLS_PER_API_KEY {
            warn!(
                max = MAX_SYMBOLS_PER_API_KEY,
                "secondary API-key symbol limit reached"
            );
        }

        let index_feed_symbols = parse_symbol_mappings(
            &get_env("INDEX_FEED_SYMBOLS", ""),
            "index",
            MAX_NON_CRYPTO_SYMBOLS,
        );
        let stock_feed_symbols = parse_symbol_mappings(
            &get_env("STOCK_FEED_SYMBOLS", ""),
            "stock",
            MAX_NON_CRYPTO_SYMBOLS,
        );
        let (primary_fx_symbols, secondary_fx_symbols, index_feed_symbols, stock_feed_symbols) =
            deduplicate_non_crypto_symbols(
                primary_fx_symbols,
                secondary_fx_symbols,
                index_feed_symbols,
                stock_feed_symbols,
            );
        let configured_non_crypto = symbol_count(
            &primary_fx_symbols,
            &secondary_fx_symbols,
            &index_feed_symbols,
            &stock_feed_symbols,
        );
        warn_if_overall_symbol_limit(configured_non_crypto);
        let (primary_fx_symbols, secondary_fx_symbols, index_feed_symbols, stock_feed_symbols) =
            cap_non_crypto_symbols(
                primary_fx_symbols,
                secondary_fx_symbols,
                index_feed_symbols,
                stock_feed_symbols,
            );

        let total_non_crypto = symbol_count(
            &primary_fx_symbols,
            &secondary_fx_symbols,
            &index_feed_symbols,
            &stock_feed_symbols,
        );
        info!(
            primary_fx = primary_fx_symbols.len(),
            secondary_fx = secondary_fx_symbols.len(),
            reference = index_feed_symbols.len(),
            equities = stock_feed_symbols.len(),
            total_non_crypto,
            crypto = crypto_symbols.len(),
            "loaded market symbol configuration"
        );
        if total_non_crypto == MAX_NON_CRYPTO_SYMBOLS {
            warn!(
                max = MAX_NON_CRYPTO_SYMBOLS,
                "overall non-crypto symbol limit reached"
            );
        }

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
            index_feed_symbols,
            stock_feed_symbols,
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
            options_sync_sec: get_env_u64("OPTIONS_POLL_SEC", 60).max(15),
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

fn parse_symbol_mappings(
    raw: &str,
    _default_asset_type: &str,
    limit: usize,
) -> Vec<MarketSymbolConfig> {
    raw.split(',')
        .filter_map(|item| {
            let item = item.trim();
            if item.is_empty() {
                return None;
            }

            let mut parts = item.split('|').map(str::trim);
            let provider_symbol = parts.next().unwrap_or_default();
            let public_symbol = parts.next().unwrap_or_default();
            let Some(asset_type) = parts.next() else {
                warn!(item = %item, "ignoring malformed market symbol mapping; expected provider|public|asset_type");
                return None;
            };

            if provider_symbol.is_empty() || public_symbol.is_empty() || asset_type.is_empty() {
                warn!(item = %item, "ignoring malformed market symbol mapping; expected provider|public|asset_type");
                return None;
            }

            Some(MarketSymbolConfig {
                provider_symbol: normalize_provider_symbol(provider_symbol),
                public_symbol: public_symbol.to_uppercase(),
                asset_type: asset_type.to_lowercase(),
            })
        })
        .take(limit)
        .collect()
}

fn normalize_provider_symbol(provider_symbol: &str) -> String {
    provider_symbol.trim().replace(':', "")
}

fn cap_non_crypto_symbols(
    primary_fx_symbols: Vec<MarketSymbolConfig>,
    secondary_fx_symbols: Vec<MarketSymbolConfig>,
    index_feed_symbols: Vec<MarketSymbolConfig>,
    stock_feed_symbols: Vec<MarketSymbolConfig>,
) -> (
    Vec<MarketSymbolConfig>,
    Vec<MarketSymbolConfig>,
    Vec<MarketSymbolConfig>,
    Vec<MarketSymbolConfig>,
) {
    let mut seen = HashSet::new();
    let mut remaining = MAX_NON_CRYPTO_SYMBOLS;

    fn keep(
        symbols: Vec<MarketSymbolConfig>,
        seen: &mut HashSet<String>,
        remaining: &mut usize,
    ) -> Vec<MarketSymbolConfig> {
        symbols
            .into_iter()
            .filter(|symbol| {
                if *remaining == 0 || !seen.insert(symbol.public_symbol.clone()) {
                    return false;
                }
                *remaining -= 1;
                true
            })
            .collect()
    }

    let primary = keep(primary_fx_symbols, &mut seen, &mut remaining);
    let secondary = keep(secondary_fx_symbols, &mut seen, &mut remaining);
    let indices = keep(index_feed_symbols, &mut seen, &mut remaining);
    let stocks = keep(stock_feed_symbols, &mut seen, &mut remaining);
    (primary, secondary, indices, stocks)
}

fn symbol_count(
    primary_fx_symbols: &[MarketSymbolConfig],
    secondary_fx_symbols: &[MarketSymbolConfig],
    index_feed_symbols: &[MarketSymbolConfig],
    stock_feed_symbols: &[MarketSymbolConfig],
) -> usize {
    primary_fx_symbols.len()
        + secondary_fx_symbols.len()
        + index_feed_symbols.len()
        + stock_feed_symbols.len()
}

fn deduplicate_non_crypto_symbols(
    primary_fx_symbols: Vec<MarketSymbolConfig>,
    secondary_fx_symbols: Vec<MarketSymbolConfig>,
    index_feed_symbols: Vec<MarketSymbolConfig>,
    stock_feed_symbols: Vec<MarketSymbolConfig>,
) -> (
    Vec<MarketSymbolConfig>,
    Vec<MarketSymbolConfig>,
    Vec<MarketSymbolConfig>,
    Vec<MarketSymbolConfig>,
) {
    let mut seen = HashSet::new();

    fn dedupe(
        symbols: Vec<MarketSymbolConfig>,
        seen: &mut HashSet<String>,
    ) -> Vec<MarketSymbolConfig> {
        symbols
            .into_iter()
            .filter(|symbol| seen.insert(symbol.public_symbol.clone()))
            .collect()
    }

    (
        dedupe(primary_fx_symbols, &mut seen),
        dedupe(secondary_fx_symbols, &mut seen),
        dedupe(index_feed_symbols, &mut seen),
        dedupe(stock_feed_symbols, &mut seen),
    )
}

fn warn_if_overall_symbol_limit(total: usize) {
    if total > MAX_NON_CRYPTO_SYMBOLS {
        warn!(
            configured = total,
            max = MAX_NON_CRYPTO_SYMBOLS,
            "configured non-crypto symbols exceed the overall limit; later entries will be dropped"
        );
    }
}

#[cfg(test)]
fn configured_non_crypto_count() -> usize {
    let primary = parse_symbol_mappings(
        "FX:EURUSD|EURUSD|forex,FX:GBPUSD|GBPUSD|forex",
        "forex",
        MAX_SYMBOLS_PER_API_KEY,
    );
    let secondary = parse_symbol_mappings(
        "FX:USDJPY|USDJPY|forex,FX:AUDUSD|AUDUSD|forex",
        "forex",
        MAX_SYMBOLS_PER_API_KEY,
    );
    let indices =
        parse_symbol_mappings("IDX:COMPOSITE|IHSG|index", "index", MAX_NON_CRYPTO_SYMBOLS);
    let stocks = Vec::new();
    let (primary, secondary, indices, stocks) =
        deduplicate_non_crypto_symbols(primary, secondary, indices, stocks);
    symbol_count(&primary, &secondary, &indices, &stocks)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_mappings_and_rejects_bare_symbols() {
        let parsed = parse_symbol_mappings("FX:EURUSD|EURUSD|forex,SPX", "index", 100);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].provider_symbol, "FXEURUSD");
        assert_eq!(parsed[0].public_symbol, "EURUSD");
        assert_eq!(parsed[0].asset_type, "forex");
    }

    #[test]
    fn caps_and_deduplicates_non_crypto_symbols() {
        let primary = (0..101)
            .map(|i| MarketSymbolConfig {
                provider_symbol: format!("NASDAQ:S{i}"),
                public_symbol: format!("S{i}"),
                asset_type: "stock".to_string(),
            })
            .collect();
        let duplicate = vec![MarketSymbolConfig {
            provider_symbol: "NASDAQ:S0".to_string(),
            public_symbol: "S0".to_string(),
            asset_type: "stock".to_string(),
        }];

        let (primary, secondary, indices, stocks) =
            cap_non_crypto_symbols(primary, duplicate, Vec::new(), Vec::new());
        assert_eq!(primary.len(), MAX_NON_CRYPTO_SYMBOLS);
        assert!(secondary.is_empty());
        assert!(indices.is_empty());
        assert!(stocks.is_empty());
    }

    #[test]
    fn two_api_keys_allow_fifty_symbols_each() {
        let (primary, secondary, indices, stocks) = cap_non_crypto_symbols(
            vec![MarketSymbolConfig {
                provider_symbol: "FX:EURUSD".to_string(),
                public_symbol: "EURUSD".to_string(),
                asset_type: "forex".to_string(),
            }],
            vec![MarketSymbolConfig {
                provider_symbol: "FX:GBPUSD".to_string(),
                public_symbol: "GBPUSD".to_string(),
                asset_type: "forex".to_string(),
            }],
            Vec::new(),
            Vec::new(),
        );
        assert_eq!(primary.len(), 1);
        assert_eq!(secondary.len(), 1);
        assert_eq!(symbol_count(&primary, &secondary, &indices, &stocks), 2);
        assert_eq!(MAX_SYMBOLS_PER_API_KEY, 50);
    }

    #[test]
    fn representative_config_includes_ihsg_and_stays_bounded() {
        assert_eq!(configured_non_crypto_count(), 5);
        assert!(configured_non_crypto_count() <= MAX_NON_CRYPTO_SYMBOLS);
    }

    #[test]
    fn secondary_key_list_is_limited_to_fifty_entries() {
        let raw = (0..60)
            .map(|i| format!("FX:S{i}|S{i}|forex"))
            .collect::<Vec<_>>()
            .join(",");
        assert_eq!(
            parse_symbol_mappings(&raw, "forex", MAX_SYMBOLS_PER_API_KEY).len(),
            MAX_SYMBOLS_PER_API_KEY
        );
    }
}
