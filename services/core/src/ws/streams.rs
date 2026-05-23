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
}
