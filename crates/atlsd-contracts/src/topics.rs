pub const MD_RAW_FINNHUB_TRADES_V1: &str = "md.raw.finnhub.trades.v1";
pub const MD_RAW_TIINGO_QUOTES_V1: &str = "md.raw.tiingo.quotes.v1";
pub const MD_NORMALIZED_TRADES_V1: &str = "md.normalized.trades.v1";
pub const MD_NORMALIZED_QUOTES_V1: &str = "md.normalized.quotes.v1";
pub const MD_CANONICAL_TICKS_V1: &str = "md.canonical.ticks.v1";
pub const MD_CANONICAL_OHLCV_1M_V1: &str = "md.canonical.ohlcv.1m.v1";
pub const MD_QUALITY_GAPS_V1: &str = "md.quality.gaps.v1";
pub const MD_REALTIME_PUBLIC_V1: &str = "md.realtime.public.v1";

pub const NEWS_RAW_ARTICLE_V1: &str = "news.raw.article.v1";
pub const NEWS_ENRICHED_ARTICLE_V1: &str = "news.enriched.article.v1";
pub const INTELLIGENCE_WHY_MOVE_GENERATED_V1: &str = "intelligence.why_move.generated.v1";
pub const INTELLIGENCE_FACTOR_UPDATED_V1: &str = "intelligence.factor.updated.v1";
pub const TENANT_ENTITLEMENT_CHANGED_V1: &str = "tenant.entitlement.changed.v1";
pub const USAGE_API_REQUESTED_V1: &str = "usage.api.requested.v1";

pub fn market_partition_key(asset_class: &str, symbol: &str) -> String {
    format!("{}:{}", asset_class.to_lowercase(), symbol.to_uppercase())
}

pub fn tenant_partition_key(tenant_id: &str) -> String {
    tenant_id.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn market_partition_key_normalizes_asset_and_symbol() {
        assert_eq!(
            market_partition_key("Commodity", "xauusd"),
            "commodity:XAUUSD"
        );
    }
}
