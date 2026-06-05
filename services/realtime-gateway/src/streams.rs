use std::collections::HashSet;

use atlsd_domain::tenant::TenantContext;
use serde_json::{json, Value};

const BASE_STREAMS: &[&str] = &[
    "all",
    "market_data",
    "forex_news",
    "stock_news",
    "calendar",
    "high_impact",
    "volatility",
    "x",
    "system",
    "geosignals",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamError {
    pub code: u16,
    pub message: String,
}

impl StreamError {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self {
            code: 400,
            message: message.into(),
        }
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self {
            code: 403,
            message: message.into(),
        }
    }

    pub fn too_many(message: impl Into<String>) -> Self {
        Self {
            code: 429,
            message: message.into(),
        }
    }
}

pub fn parse_stream(raw: &str) -> Result<String, StreamError> {
    let stream = raw.trim();
    if stream.is_empty() {
        return Err(StreamError::bad_request("Stream name cannot be empty"));
    }

    let lower = stream.to_lowercase();
    if BASE_STREAMS.contains(&lower.as_str()) {
        return Ok(lower);
    }

    if let Some(symbol) = stream.strip_prefix("market_data:") {
        let symbol = normalize_symbol(symbol);
        if symbol.is_empty() {
            return Err(StreamError::bad_request("Market stream requires a symbol"));
        }
        return Ok(format!("market_data:{symbol}"));
    }

    if let Some(username) = lower.strip_prefix("x:") {
        let username = normalize_username(username);
        if username.is_empty() {
            return Err(StreamError::bad_request("X stream requires a username"));
        }
        return Ok(format!("x:{username}"));
    }

    if let Some(rest) = lower.strip_prefix("geosignals:") {
        return parse_geosignals_stream(rest);
    }

    Err(StreamError::bad_request(format!(
        "Unknown stream: {stream}"
    )))
}

pub fn normalize_streams<I, S>(streams: I) -> Result<HashSet<String>, StreamError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    streams
        .into_iter()
        .map(|s| parse_stream(s.as_ref()))
        .collect()
}

pub fn candidate_streams(channel: &str, data: &Value) -> HashSet<String> {
    let mut streams = HashSet::from(["all".to_string(), channel.to_string()]);

    if channel == "market_data" {
        if let Some(symbol) = data
            .get("tick")
            .and_then(|tick| tick.get("symbol"))
            .and_then(|symbol| symbol.as_str())
            .map(normalize_symbol)
            .filter(|symbol| !symbol.is_empty())
        {
            streams.insert(format!("market_data:{symbol}"));
        }
    }

    if channel == "x" {
        if let Some(username) = data
            .get("post")
            .and_then(|post| post.get("author_username"))
            .and_then(|username| username.as_str())
            .map(normalize_username)
            .filter(|username| !username.is_empty())
        {
            streams.insert(format!("x:{username}"));
        }
    }

    if channel == "geosignals" {
        if let Some(country) = data
            .get("country")
            .and_then(|c| c.as_str())
            .map(normalize_slug)
            .filter(|s| !s.is_empty())
        {
            streams.insert(format!("geosignals:country:{country}"));
        }

        if let Some(region) = data
            .get("region")
            .and_then(|r| r.as_str())
            .map(normalize_slug)
            .filter(|s| !s.is_empty())
        {
            streams.insert(format!("geosignals:region:{region}"));
        }

        if let Some(category) = data
            .get("category")
            .and_then(|c| c.as_str())
            .map(normalize_slug)
            .filter(|s| !s.is_empty())
        {
            streams.insert(format!("geosignals:category:{category}"));
        }

        if let Some(assets) = data.get("affected_assets").and_then(|a| a.as_array()) {
            for asset in assets {
                if let Some(symbol) = asset
                    .as_str()
                    .map(normalize_symbol)
                    .filter(|s| !s.is_empty())
                {
                    streams.insert(format!("geosignals:asset:{symbol}"));
                }
            }
        }
    }

    streams
}

pub fn event_stream(channel: &str, data: &Value) -> String {
    if channel == "market_data" {
        if let Some(symbol) = data
            .get("tick")
            .and_then(|tick| tick.get("symbol"))
            .and_then(|symbol| symbol.as_str())
            .map(normalize_symbol)
            .filter(|symbol| !symbol.is_empty())
        {
            return format!("market_data:{symbol}");
        }
    }

    if channel == "x" {
        if let Some(username) = data
            .get("post")
            .and_then(|post| post.get("author_username"))
            .and_then(|username| username.as_str())
            .map(normalize_username)
            .filter(|username| !username.is_empty())
        {
            return format!("x:{username}");
        }
    }

    channel.to_string()
}

pub fn validate_subscription_change(
    ctx: Option<&TenantContext>,
    current: &HashSet<String>,
    additions: &HashSet<String>,
) -> Result<(), StreamError> {
    let Some(ctx) = ctx else {
        return Ok(());
    };

    if ctx.is_admin {
        return Ok(());
    }

    let mut next = current.clone();
    next.extend(additions.iter().cloned());

    let market_symbols: HashSet<String> = next
        .iter()
        .filter_map(|stream| stream.strip_prefix("market_data:").map(str::to_string))
        .collect();

    if market_symbols.len() > ctx.tv_symbols_max.max(0) as usize {
        return Err(StreamError::too_many(format!(
            "Market symbol subscription limit reached for your plan ({})",
            ctx.tv_symbols_max
        )));
    }

    if !ctx.tv_symbols.is_empty() && !market_symbols.is_subset(&ctx.tv_symbols) {
        return Err(StreamError::forbidden(
            "Requested market symbol is not allowed by your plan",
        ));
    }

    let x_usernames: HashSet<String> = next
        .iter()
        .filter_map(|stream| stream.strip_prefix("x:").map(str::to_string))
        .collect();

    if x_usernames.len() > ctx.x_usernames_max.max(0) as usize {
        return Err(StreamError::too_many(format!(
            "X username subscription limit reached for your plan ({})",
            ctx.x_usernames_max
        )));
    }

    if !ctx.x_usernames.is_empty() && !x_usernames.is_subset(&ctx.x_usernames) {
        return Err(StreamError::forbidden(
            "Requested X username is not allowed by your plan",
        ));
    }

    if ctx.plan == "free" {
        for stream in &next {
            if matches!(
                stream.as_str(),
                "stock_news" | "calendar" | "high_impact" | "volatility" | "x"
            ) || stream.starts_with("x:")
            {
                return Err(StreamError::forbidden(
                    "This stream is not available on the free plan",
                ));
            }
        }
    }

    Ok(())
}

pub fn error_response(error: &StreamError, id: Option<Value>) -> Value {
    json!({
        "error": {
            "code": error.code,
            "msg": error.message,
        },
        "id": id.unwrap_or(Value::Null),
    })
}

fn normalize_symbol(symbol: &str) -> String {
    symbol.trim().to_uppercase()
}

fn normalize_username(username: &str) -> String {
    username.trim().trim_start_matches('@').to_lowercase()
}

fn normalize_slug(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .replace('_', "-")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
}

fn parse_geosignals_stream(rest: &str) -> Result<String, StreamError> {
    if let Some(slug) = rest.strip_prefix("country:") {
        let slug = normalize_slug(slug);
        if slug.is_empty() {
            return Err(StreamError::bad_request(
                "Geosignals country stream requires a country",
            ));
        }
        return Ok(format!("geosignals:country:{slug}"));
    }

    if let Some(slug) = rest.strip_prefix("region:") {
        let slug = normalize_slug(slug);
        if slug.is_empty() {
            return Err(StreamError::bad_request(
                "Geosignals region stream requires a region",
            ));
        }
        return Ok(format!("geosignals:region:{slug}"));
    }

    if let Some(slug) = rest.strip_prefix("category:") {
        let slug = normalize_slug(slug);
        if slug.is_empty() {
            return Err(StreamError::bad_request(
                "Geosignals category stream requires a category",
            ));
        }
        return Ok(format!("geosignals:category:{slug}"));
    }

    if let Some(symbol) = rest.strip_prefix("asset:") {
        let symbol = normalize_symbol(symbol);
        if symbol.is_empty() {
            return Err(StreamError::bad_request(
                "Geosignals asset stream requires a symbol",
            ));
        }
        return Ok(format!("geosignals:asset:{symbol}"));
    }

    Err(StreamError::bad_request(format!(
        "Unknown geosignals stream: {rest}"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_stream_normalizes_market_symbol() {
        assert_eq!(
            parse_stream("market_data:xauusd").unwrap(),
            "market_data:XAUUSD"
        );
    }

    #[test]
    fn parse_stream_normalizes_x_username() {
        assert_eq!(
            parse_stream("x:@FederalReserve").unwrap(),
            "x:federalreserve"
        );
    }

    #[test]
    fn parse_stream_rejects_unknown_stream() {
        assert!(parse_stream("unknown").is_err());
    }

    // Geosignals country stream tests
    #[test]
    fn parse_stream_normalizes_geosignals_country() {
        assert_eq!(
            parse_stream("geosignals:country:United States").unwrap(),
            "geosignals:country:united-states"
        );
    }

    #[test]
    fn parse_stream_geosignals_country_lowercase() {
        assert_eq!(
            parse_stream("geosignals:country:FRANCE").unwrap(),
            "geosignals:country:france"
        );
    }

    #[test]
    fn parse_stream_geosignals_country_multi_word() {
        assert_eq!(
            parse_stream("geosignals:country:New Zealand").unwrap(),
            "geosignals:country:new-zealand"
        );
    }

    #[test]
    fn parse_stream_geosignals_country_empty_rejects() {
        assert!(parse_stream("geosignals:country:").is_err());
    }

    // Geosignals region stream tests
    #[test]
    fn parse_stream_normalizes_geosignals_region() {
        assert_eq!(
            parse_stream("geosignals:region:North America").unwrap(),
            "geosignals:region:north-america"
        );
    }

    #[test]
    fn parse_stream_geosignals_region_lowercase() {
        assert_eq!(
            parse_stream("geosignals:region:EUROPE").unwrap(),
            "geosignals:region:europe"
        );
    }

    #[test]
    fn parse_stream_geosignals_region_empty_rejects() {
        assert!(parse_stream("geosignals:region:").is_err());
    }

    // Geosignals category stream tests
    #[test]
    fn parse_stream_normalizes_geosignals_category() {
        assert_eq!(
            parse_stream("geosignals:category:Natural Disaster").unwrap(),
            "geosignals:category:natural-disaster"
        );
    }

    #[test]
    fn parse_stream_geosignals_category_lowercase() {
        assert_eq!(
            parse_stream("geosignals:category:POLITICAL UNREST").unwrap(),
            "geosignals:category:political-unrest"
        );
    }

    #[test]
    fn parse_stream_geosignals_category_empty_rejects() {
        assert!(parse_stream("geosignals:category:").is_err());
    }

    // Geosignals asset stream tests
    #[test]
    fn parse_stream_normalizes_geosignals_asset() {
        assert_eq!(
            parse_stream("geosignals:asset:aapl").unwrap(),
            "geosignals:asset:AAPL"
        );
    }

    #[test]
    fn parse_stream_geosignals_asset_uppercase() {
        assert_eq!(
            parse_stream("geosignals:asset:MSFT").unwrap(),
            "geosignals:asset:MSFT"
        );
    }

    #[test]
    fn parse_stream_geosignals_asset_mixed_case() {
        assert_eq!(
            parse_stream("geosignals:asset:GoOgL").unwrap(),
            "geosignals:asset:GOOGL"
        );
    }

    #[test]
    fn parse_stream_geosignals_asset_empty_rejects() {
        assert!(parse_stream("geosignals:asset:").is_err());
    }

    // Geosignals base stream
    #[test]
    fn parse_stream_geosignals_base() {
        assert_eq!(parse_stream("geosignals").unwrap(), "geosignals");
    }

    #[test]
    fn parse_stream_geosignals_base_case_insensitive() {
        assert_eq!(parse_stream("GEOSIGNALS").unwrap(), "geosignals");
    }

    // Candidate streams tests for geosignals
    #[test]
    fn candidate_streams_geosignals_includes_base() {
        let data = json!({});
        let streams = candidate_streams("geosignals", &data);
        assert!(streams.contains("all"));
        assert!(streams.contains("geosignals"));
    }

    #[test]
    fn candidate_streams_geosignals_country() {
        let data = json!({
            "country": "United States"
        });
        let streams = candidate_streams("geosignals", &data);
        assert!(streams.contains("geosignals:country:united-states"));
    }

    #[test]
    fn candidate_streams_geosignals_region() {
        let data = json!({
            "region": "North America"
        });
        let streams = candidate_streams("geosignals", &data);
        assert!(streams.contains("geosignals:region:north-america"));
    }

    #[test]
    fn candidate_streams_geosignals_category() {
        let data = json!({
            "category": "Natural Disaster"
        });
        let streams = candidate_streams("geosignals", &data);
        assert!(streams.contains("geosignals:category:natural-disaster"));
    }

    #[test]
    fn category_streams_normalize_snake_case_to_slug() {
        assert_eq!(
            parse_stream("geosignals:category:supply_chain").unwrap(),
            "geosignals:category:supply-chain"
        );
        let data = json!({
            "category": "supply_chain"
        });
        let streams = candidate_streams("geosignals", &data);
        assert!(streams.contains("geosignals:category:supply-chain"));
    }

    #[test]
    fn candidate_streams_geosignals_single_asset() {
        let data = json!({
            "affected_assets": ["AAPL"]
        });
        let streams = candidate_streams("geosignals", &data);
        assert!(streams.contains("geosignals:asset:AAPL"));
    }

    #[test]
    fn candidate_streams_geosignals_multiple_assets() {
        let data = json!({
            "affected_assets": ["aapl", "MSFT", "googl"]
        });
        let streams = candidate_streams("geosignals", &data);
        assert!(streams.contains("geosignals:asset:AAPL"));
        assert!(streams.contains("geosignals:asset:MSFT"));
        assert!(streams.contains("geosignals:asset:GOOGL"));
    }

    #[test]
    fn candidate_streams_geosignals_all_fields() {
        let data = json!({
            "country": "France",
            "region": "Europe",
            "category": "Political Unrest",
            "affected_assets": ["bnp", "TCS"]
        });
        let streams = candidate_streams("geosignals", &data);
        assert!(streams.contains("all"));
        assert!(streams.contains("geosignals"));
        assert!(streams.contains("geosignals:country:france"));
        assert!(streams.contains("geosignals:region:europe"));
        assert!(streams.contains("geosignals:category:political-unrest"));
        assert!(streams.contains("geosignals:asset:BNP"));
        assert!(streams.contains("geosignals:asset:TCS"));
        assert_eq!(streams.len(), 7);
    }

    #[test]
    fn candidate_streams_geosignals_ignores_empty_assets() {
        let data = json!({
            "affected_assets": ["AAPL", "", "MSFT"]
        });
        let streams = candidate_streams("geosignals", &data);
        assert!(streams.contains("geosignals:asset:AAPL"));
        assert!(streams.contains("geosignals:asset:MSFT"));
        // Should not contain empty asset stream
        assert!(!streams.iter().any(|s| s == "geosignals:asset:"));
    }

    #[test]
    fn candidate_streams_geosignals_ignores_null_fields() {
        let data = json!({
            "country": null,
            "region": "Europe",
            "category": null
        });
        let streams = candidate_streams("geosignals", &data);
        assert!(streams.contains("geosignals:region:europe"));
        assert!(!streams.iter().any(|s| s.starts_with("geosignals:country:")));
        assert!(!streams
            .iter()
            .any(|s| s.starts_with("geosignals:category:")));
    }

    #[test]
    fn candidate_streams_geosignals_empty_assets_array() {
        let data = json!({
            "affected_assets": []
        });
        let streams = candidate_streams("geosignals", &data);
        assert!(streams.contains("all"));
        assert!(streams.contains("geosignals"));
        // Only base streams
        assert_eq!(streams.len(), 2);
    }

    #[test]
    fn candidate_streams_geosignals_country_with_spaces() {
        let data = json!({
            "country": "  United Arab Emirates  "
        });
        let streams = candidate_streams("geosignals", &data);
        assert!(streams.contains("geosignals:country:united-arab-emirates"));
    }

    #[test]
    fn candidate_streams_non_geosignals_channel() {
        let data = json!({
            "country": "France",
            "region": "Europe"
        });
        let streams = candidate_streams("market_data", &data);
        // Should not include geosignals streams for non-geosignals channel
        assert!(streams.contains("all"));
        assert!(streams.contains("market_data"));
        assert!(!streams.iter().any(|s| s.starts_with("geosignals:")));
    }
}
