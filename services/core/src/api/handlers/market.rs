use axum::{
    extract::{Path, Query, State},
    Json,
};
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::api::state::AppState;
use crate::clickhouse::{LatestPriceTick, SpikeCandidate};
use crate::ingestion_subscriber::{self, CachedPrice};

static HTTP_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .unwrap_or_default()
});

type LatestPriceRow = (
    String,
    f64,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    String,
    String,
    Option<chrono::DateTime<chrono::Utc>>,
);

fn cached_price_from_row(row: LatestPriceRow) -> CachedPrice {
    let (symbol, price, bid, ask, volume, source, asset_type, received_at) = row;
    CachedPrice {
        symbol,
        price,
        bid,
        ask,
        volume,
        source,
        asset_type,
        received_at: received_at.map(|dt| dt.to_rfc3339()),
        updated_at: None,
    }
}

fn price_json(p: &CachedPrice) -> Value {
    json!({
        "symbol": p.symbol,
        "price": p.price,
        "bid": p.bid,
        "ask": p.ask,
        "volume": p.volume,
        "source": p.source,
        "asset_type": p.asset_type,
        "received_at": p.received_at,
    })
}

fn cached_price_from_clickhouse(row: LatestPriceTick) -> CachedPrice {
    CachedPrice {
        symbol: row.symbol,
        price: row.price,
        bid: row.bid,
        ask: row.ask,
        volume: Some(row.volume),
        source: row.source,
        asset_type: row.asset_type,
        received_at: Some(row.received_at),
        updated_at: None,
    }
}

async fn load_clickhouse_latest_prices(state: &AppState) -> Vec<CachedPrice> {
    let Some(clickhouse) = &state.clickhouse else {
        return Vec::new();
    };

    match clickhouse.latest_prices().await {
        Ok(rows) => rows.into_iter().map(cached_price_from_clickhouse).collect(),
        Err(err) => {
            tracing::warn!(error = %err, "failed to load latest market prices from ClickHouse");
            Vec::new()
        }
    }
}

async fn load_clickhouse_latest_price(state: &AppState, symbol: &str) -> Option<CachedPrice> {
    let clickhouse = state.clickhouse.as_ref()?;
    match clickhouse.latest_price(symbol).await {
        Ok(Some(row)) => Some(cached_price_from_clickhouse(row)),
        Ok(None) => None,
        Err(err) => {
            tracing::warn!(error = %err, symbol = %symbol, "failed to load latest market price from ClickHouse");
            None
        }
    }
}

async fn load_latest_prices(pool: &sqlx::PgPool) -> Result<Vec<CachedPrice>, sqlx::Error> {
    let rows: Vec<LatestPriceRow> = sqlx::query_as(
        "SELECT symbol, price, bid, ask, volume, source, asset_type, received_at \
         FROM market.market_latest_prices \
         ORDER BY symbol",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(cached_price_from_row).collect())
}

async fn load_latest_price(
    pool: &sqlx::PgPool,
    symbol: &str,
) -> Result<Option<CachedPrice>, sqlx::Error> {
    let row: Option<LatestPriceRow> = sqlx::query_as(
        "SELECT symbol, price, bid, ask, volume, source, asset_type, received_at \
         FROM market.market_latest_prices \
         WHERE symbol = $1",
    )
    .bind(symbol)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(cached_price_from_row))
}

async fn latest_price_history_fallback(pool: &sqlx::PgPool, symbol: &str) -> Vec<Value> {
    match load_latest_price(pool, symbol).await {
        Ok(Some(p)) if p.price > 0.0 => {
            let now = chrono::Utc::now().timestamp();
            let start = now - (119 * 60);
            (0..120)
                .map(|i| {
                    json!({
                        "time": start + (i * 60),
                        "value": p.price,
                        "source": "last_known"
                    })
                })
                .collect()
        }
        Ok(_) => Vec::new(),
        Err(err) => {
            tracing::warn!(error = %err, symbol = %symbol, "failed to load latest price for history fallback");
            Vec::new()
        }
    }
}

fn tradingview_symbol(symbol: &str) -> String {
    let sym = symbol.trim().to_uppercase();
    match sym.as_str() {
        "XAUUSD" => "OANDA:XAUUSD".to_string(),
        "SPX" => "SP:SPX".to_string(),
        "DXY" => "TVC:DXY".to_string(),
        _ if sym.len() == 6 && sym.chars().all(|c| c.is_ascii_uppercase()) => format!("FX:{sym}"),
        _ => sym,
    }
}

fn encode_symbol(symbol: &str) -> String {
    symbol.replace(':', "%3A")
}

fn parse_reference_history(value: &Value) -> Vec<Value> {
    let mut history = parse_bar_array(value)
        .or_else(|| value.get("bars").and_then(parse_bar_array))
        .or_else(|| parse_compact_history(value))
        .or_else(|| parse_yahoo_history(value))
        .unwrap_or_default();

    let limit = 120;
    if history.len() > limit {
        history = history.split_off(history.len() - limit);
    }
    history
}

fn parse_bar_array(value: &Value) -> Option<Vec<Value>> {
    let bars = value.as_array()?;
    let history: Vec<Value> = bars
        .iter()
        .filter_map(|bar| {
            let time = bar
                .get("time")
                .or_else(|| bar.get("timestamp"))
                .and_then(as_timestamp)?;
            let close = bar
                .get("close")
                .or_else(|| bar.get("value"))
                .and_then(as_price)?;
            Some(json!({ "time": time, "value": close }))
        })
        .collect();

    if history.is_empty() {
        None
    } else {
        Some(history)
    }
}

fn parse_compact_history(value: &Value) -> Option<Vec<Value>> {
    let timestamps = value.get("t")?.as_array()?;
    let closes = value.get("c")?.as_array()?;
    let len = timestamps.len().min(closes.len());
    let history: Vec<Value> = (0..len)
        .filter_map(|i| {
            Some(json!({
                "time": as_timestamp(&timestamps[i])?,
                "value": as_price(&closes[i])?,
            }))
        })
        .collect();

    if history.is_empty() {
        None
    } else {
        Some(history)
    }
}

fn parse_yahoo_history(value: &Value) -> Option<Vec<Value>> {
    let result = value.get("chart")?.get("result")?.get(0)?;
    let timestamps = result.get("timestamp")?.as_array()?;
    let closes = result
        .get("indicators")?
        .get("quote")?
        .get(0)?
        .get("close")?
        .as_array()?;
    let len = timestamps.len().min(closes.len());
    let history: Vec<Value> = (0..len)
        .filter_map(|i| {
            Some(json!({
                "time": as_timestamp(&timestamps[i])?,
                "value": as_price(&closes[i])?,
            }))
        })
        .collect();

    if history.is_empty() {
        None
    } else {
        Some(history)
    }
}

fn as_timestamp(value: &Value) -> Option<i64> {
    match value {
        Value::Number(n) => n
            .as_i64()
            .map(|ts| if ts > 10_000_000_000 { ts / 1000 } else { ts }),
        Value::String(s) => {
            s.parse::<i64>()
                .ok()
                .map(|ts| if ts > 10_000_000_000 { ts / 1000 } else { ts })
        }
        _ => None,
    }
}

fn as_price(value: &Value) -> Option<f64> {
    match value {
        Value::Number(n) => n.as_f64().filter(|price| *price > 0.0),
        Value::String(s) => s.parse::<f64>().ok().filter(|price| *price > 0.0),
        _ => None,
    }
}

fn history_is_usable(history: &[Value], now: chrono::DateTime<chrono::Utc>) -> bool {
    if history.len() < 5 {
        return false;
    }

    let last_time = history
        .last()
        .and_then(|point| point.get("time"))
        .and_then(as_timestamp);
    if last_time.is_some_and(|ts| now.timestamp() - ts > 6 * 60 * 60) {
        return false;
    }

    let prices: Vec<f64> = history
        .iter()
        .filter_map(|point| point.get("value").and_then(as_price))
        .collect();
    if prices.len() < 5 {
        return false;
    }

    let min = prices.iter().copied().fold(f64::INFINITY, f64::min);
    let max = prices.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let latest = prices.last().copied().unwrap_or(0.0).abs().max(1.0);
    ((max - min) / latest) >= 0.00001
}

pub async fn list_prices(State(state): State<AppState>) -> Json<Value> {
    let mut prices = ingestion_subscriber::get_all_prices();

    if prices.is_empty() {
        match load_latest_prices(&state.db).await {
            Ok(db_prices) => {
                for price in &db_prices {
                    ingestion_subscriber::set_price(price.clone());
                }
                prices = db_prices;
            }
            Err(err) => {
                tracing::warn!(error = %err, "failed to load latest market prices from database");
            }
        }
    }

    if prices.is_empty() {
        let clickhouse_prices = load_clickhouse_latest_prices(&state).await;
        for price in &clickhouse_prices {
            ingestion_subscriber::set_price(price.clone());
        }
        prices = clickhouse_prices;
    }

    let items: Vec<Value> = prices.iter().map(price_json).collect();

    Json(json!({
        "items": items,
        "total": items.len(),
    }))
}

pub async fn get_price(State(state): State<AppState>, Path(symbol): Path<String>) -> Json<Value> {
    if let Some(p) = ingestion_subscriber::get_price(&symbol) {
        return Json(price_json(&p));
    }

    let sym = symbol.to_uppercase();
    match load_latest_price(&state.db, &sym).await {
        Ok(Some(p)) => {
            ingestion_subscriber::set_price(p.clone());
            Json(price_json(&p))
        }
        Ok(None) => {
            if let Some(p) = load_clickhouse_latest_price(&state, &sym).await {
                ingestion_subscriber::set_price(p.clone());
                Json(price_json(&p))
            } else {
                Json(json!({
                    "error": format!("Symbol '{}' not found in price cache or latest price store. Available symbols can be retrieved from GET /api/v1/market/prices", sym),
                }))
            }
        }
        Err(err) => {
            tracing::warn!(error = %err, symbol = %sym, "failed to load latest market price from database");
            if let Some(p) = load_clickhouse_latest_price(&state, &sym).await {
                ingestion_subscriber::set_price(p.clone());
                Json(price_json(&p))
            } else {
                Json(json!({
                    "error": format!("Symbol '{}' not found in price cache. Available symbols can be retrieved from GET /api/v1/market/prices", sym),
                }))
            }
        }
    }
}

#[derive(Deserialize)]
pub struct SpikesQuery {
    pub window: Option<String>,
}

fn spike_window_minutes(window: Option<&str>) -> u32 {
    match window.unwrap_or("5m") {
        "15m" => 15,
        "30m" => 30,
        "1h" => 60,
        _ => 5,
    }
}

fn spike_threshold(symbol: &str, asset_type: &str) -> f64 {
    let symbol = symbol.to_uppercase();
    if symbol == "DXY" {
        0.12
    } else if symbol == "XAUUSD" {
        0.25
    } else if symbol == "SPX" || asset_type.eq_ignore_ascii_case("index") {
        0.30
    } else if symbol.ends_with("USDT") || asset_type.eq_ignore_ascii_case("crypto") {
        0.80
    } else if asset_type.eq_ignore_ascii_case("forex") || symbol.len() == 6 {
        0.15
    } else {
        0.50
    }
}

fn spike_severity(move_pct: f64, threshold: f64) -> &'static str {
    if move_pct.abs() >= threshold * 2.0 {
        "high"
    } else {
        "medium"
    }
}

fn spike_json(candidate: SpikeCandidate, window: &str) -> Option<Value> {
    let threshold = spike_threshold(&candidate.symbol, &candidate.asset_type);
    if candidate.move_pct.abs() < threshold {
        return None;
    }

    Some(json!({
        "symbol": candidate.symbol,
        "asset_type": candidate.asset_type,
        "window": window,
        "latest_price": candidate.latest_price,
        "baseline_price": candidate.baseline_price,
        "move_pct": candidate.move_pct,
        "direction": if candidate.move_pct >= 0.0 { "up" } else { "down" },
        "severity": spike_severity(candidate.move_pct, threshold),
        "threshold_pct": threshold,
        "tick_count": candidate.tick_count,
        "latest_at": candidate.latest_at,
    }))
}

pub async fn volatility_spikes(
    State(state): State<AppState>,
    Query(query): Query<SpikesQuery>,
) -> Json<Value> {
    let window_minutes = spike_window_minutes(query.window.as_deref());
    let window = format!("{window_minutes}m");
    let items: Vec<Value> = if let Some(clickhouse) = &state.clickhouse {
        match clickhouse.spike_candidates(window_minutes).await {
            Ok(rows) => rows
                .into_iter()
                .filter_map(|candidate| spike_json(candidate, &window))
                .collect(),
            Err(err) => {
                tracing::warn!(error = %err, window = %window, "failed to load volatility spike candidates");
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };

    Json(json!({
        "items": items,
        "total": items.len(),
        "window": window,
        "generated_at": chrono::Utc::now().to_rfc3339(),
    }))
}

#[derive(Deserialize)]
pub struct WhyQuery {
    pub window: Option<String>,
    pub lookback_minutes: Option<u32>,
    pub refresh: Option<bool>,
}

struct NewsCause {
    kind: &'static str,
    title: String,
    summary: Option<String>,
    source_name: Option<String>,
    url: Option<String>,
    published_at: Option<chrono::DateTime<chrono::Utc>>,
    processed_at: Option<chrono::DateTime<chrono::Utc>>,
    sentiment: Option<String>,
    impact_level: Option<String>,
    searchable: String,
}

type ForexCauseRow = (
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<chrono::DateTime<chrono::Utc>>,
    Option<chrono::DateTime<chrono::Utc>>,
    Option<String>,
    Option<String>,
    String,
);

type StockCauseRow = (
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<chrono::DateTime<chrono::Utc>>,
);

fn normalize_market_symbol(symbol: &str) -> String {
    symbol
        .trim()
        .to_uppercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect()
}

fn symbol_terms(symbol: &str) -> Vec<String> {
    let symbol = normalize_market_symbol(symbol);
    let mut terms = vec![symbol.clone()];

    match symbol.as_str() {
        "XAUUSD" => terms.extend(
            ["XAU", "GOLD", "EMAS", "USD", "FED", "INFLATION", "YIELD"].map(str::to_string),
        ),
        "DXY" => terms
            .extend(["DOLLAR", "USD", "GREENBACK", "FED", "TREASURY", "YIELD"].map(str::to_string)),
        "SPX" => terms.extend(
            [
                "SPX",
                "S&P 500",
                "S&P500",
                "US500",
                "STOCK",
                "EQUITY",
                "FED",
                "INFLATION",
            ]
            .map(str::to_string),
        ),
        "BTCUSDT" => {
            terms.extend(["BTC", "BITCOIN", "CRYPTO", "KRIPTO", "USDT"].map(str::to_string))
        }
        "ETHUSDT" => {
            terms.extend(["ETH", "ETHEREUM", "CRYPTO", "KRIPTO", "USDT"].map(str::to_string))
        }
        _ => {
            if symbol.ends_with("USDT") {
                terms.extend(["CRYPTO", "KRIPTO", "USDT"].map(str::to_string));
                terms.push(symbol.trim_end_matches("USDT").to_string());
            } else if symbol.len() == 6 {
                terms.push(symbol[0..3].to_string());
                terms.push(symbol[3..6].to_string());
            }
        }
    }

    terms.sort();
    terms.dedup();
    terms
}

fn matched_terms(searchable: &str, terms: &[String]) -> Vec<String> {
    let haystack = searchable.to_uppercase();
    terms
        .iter()
        .filter(|term| haystack.contains(term.as_str()))
        .cloned()
        .collect()
}

fn symbol_driver_terms(symbol: &str) -> Vec<&'static str> {
    match normalize_market_symbol(symbol).as_str() {
        "XAUUSD" => vec![
            "USD",
            "real yields",
            "Fed policy",
            "inflation",
            "safe haven",
        ],
        "DXY" => vec!["USD", "Fed policy", "Treasury yields", "inflation"],
        "SPX" => vec!["risk sentiment", "earnings", "Fed policy", "inflation"],
        "BTCUSDT" | "ETHUSDT" => vec![
            "crypto risk appetite",
            "USD liquidity",
            "ETF flow",
            "macro risk",
        ],
        _ => vec!["symbol news", "sentiment", "macro risk", "USD liquidity"],
    }
}

fn cross_asset_relationship(symbol: &str, other_symbol: &str, other_move_pct: f64) -> &'static str {
    let symbol = normalize_market_symbol(symbol);
    let other = normalize_market_symbol(other_symbol);
    match (
        symbol.as_str(),
        other.as_str(),
        other_move_pct.is_sign_positive(),
    ) {
        ("XAUUSD", "DXY", false) => "DXY weakness supports gold strength",
        ("XAUUSD", "DXY", true) => "DXY strength conflicts with gold strength",
        ("DXY", "XAUUSD", false) => "Gold weakness can align with USD strength",
        ("DXY", "XAUUSD", true) => "Gold strength may signal USD pressure",
        (_, "SPX", false) => "Equity weakness suggests risk-off pressure",
        (_, "SPX", true) => "Equity strength suggests risk-on tone",
        (_, "BTCUSDT", true) | (_, "ETHUSDT", true) => "Crypto strength suggests risk appetite",
        (_, "BTCUSDT", false) | (_, "ETHUSDT", false) => "Crypto weakness suggests risk caution",
        _ => "Same-window market movement",
    }
}

fn sentiment_aligns(sentiment: Option<&str>, direction: &str) -> bool {
    matches!(
        (sentiment.map(str::to_lowercase).as_deref(), direction),
        (Some("positive" | "bullish"), "up") | (Some("negative" | "bearish"), "down")
    )
}

fn score_cause(
    cause: &NewsCause,
    terms: &[String],
    direction: &str,
    latest_at: chrono::DateTime<chrono::Utc>,
) -> (f64, Vec<String>) {
    let matches = matched_terms(&cause.searchable, terms);
    if matches.is_empty() {
        return (0.0, matches);
    }

    let mut score = 10.0 + matches.len() as f64 * 6.0;
    if matches.iter().any(|term| term.len() >= 6) {
        score += 12.0;
    }
    if cause
        .impact_level
        .as_deref()
        .is_some_and(|impact| impact.eq_ignore_ascii_case("high"))
    {
        score += 10.0;
    }
    if sentiment_aligns(cause.sentiment.as_deref(), direction) {
        score += 8.0;
    }

    let event_at = cause
        .processed_at
        .or(cause.published_at)
        .unwrap_or(latest_at);
    let minutes = (latest_at - event_at).num_minutes().unsigned_abs();
    if minutes <= 30 {
        score += 10.0;
    } else if minutes <= 120 {
        score += 5.0;
    }

    (score, matches)
}

fn confidence_for(top_score: f64, cause_count: usize) -> &'static str {
    if top_score >= 45.0 && cause_count >= 2 {
        "high"
    } else if top_score >= 25.0 {
        "medium"
    } else {
        "low"
    }
}

fn why_summary(
    symbol: &str,
    candidate: Option<&SpikeCandidate>,
    cause_count: usize,
    confidence: &str,
) -> String {
    match candidate {
        Some(candidate) if cause_count > 0 => format!(
            "{symbol} moved {} {:.2}% over the selected window with {cause_count} relevant news catalyst(s) nearby. Confidence is {confidence}.",
            if candidate.move_pct >= 0.0 { "up" } else { "down" },
            candidate.move_pct.abs()
        ),
        Some(candidate) => format!(
            "{symbol} moved {} {:.2}% over the selected window, but no matching news catalyst was found in the lookback window.",
            if candidate.move_pct >= 0.0 { "up" } else { "down" },
            candidate.move_pct.abs()
        ),
        None => format!("No recent market move context is available for {symbol}."),
    }
}

async fn load_news_causes(
    db: &sqlx::PgPool,
    since: chrono::DateTime<chrono::Utc>,
    until: chrono::DateTime<chrono::Utc>,
) -> Result<Vec<NewsCause>, sqlx::Error> {
    let forex_rows: Vec<ForexCauseRow> = sqlx::query_as(
        "SELECT a.original_title, a.summary, COALESCE(s.name, 'Unknown') AS source_name, a.original_url, \
         a.published_at, a.processed_at, an.sentiment, an.impact_level, COALESCE(an.currency_pairs, '') \
         FROM news.forex_news_articles a \
         LEFT JOIN news.forex_news_sources s ON a.source_id = s.id \
         LEFT JOIN news.forex_news_analyses an ON a.id = an.article_id \
         WHERE a.is_processed = TRUE AND COALESCE(a.processed_at, a.published_at, a.created_at) BETWEEN $1 AND $2 \
         ORDER BY COALESCE(a.processed_at, a.published_at, a.created_at) DESC \
         LIMIT 100",
    )
    .bind(since)
    .bind(until)
    .fetch_all(db)
    .await?;

    let stock_rows: Vec<StockCauseRow> = sqlx::query_as(
        "SELECT title, summary, source_name, tickers, sentiment, impact_level, processed_at \
         FROM news.stock_news \
         WHERE is_processed = TRUE AND COALESCE(processed_at, created_at) BETWEEN $1 AND $2 \
         ORDER BY COALESCE(processed_at, created_at) DESC \
         LIMIT 100",
    )
    .bind(since)
    .bind(until)
    .fetch_all(db)
    .await?;

    let mut causes = Vec::with_capacity(forex_rows.len() + stock_rows.len());
    for row in forex_rows {
        let searchable = format!("{} {} {}", row.0, row.1.clone().unwrap_or_default(), row.8);
        causes.push(NewsCause {
            kind: "forex_news",
            title: row.0,
            summary: row.1,
            source_name: row.2,
            url: row.3,
            published_at: row.4,
            processed_at: row.5,
            sentiment: row.6,
            impact_level: row.7,
            searchable,
        });
    }
    for row in stock_rows {
        let searchable = format!(
            "{} {} {}",
            row.0,
            row.1.clone().unwrap_or_default(),
            row.3.clone().unwrap_or_default()
        );
        causes.push(NewsCause {
            kind: "stock_news",
            title: row.0,
            summary: row.1,
            source_name: row.2,
            url: None,
            published_at: row.6,
            processed_at: row.6,
            sentiment: row.4,
            impact_level: row.5,
            searchable,
        });
    }

    Ok(causes)
}

fn evidence_hash(evidence: &Value) -> String {
    let serialized = serde_json::to_string(evidence).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(serialized.as_bytes());
    hex::encode(hasher.finalize())
}

fn with_cache_metadata(mut response: Value, cache_status: &str, engine_status: &str) -> Value {
    if let Some(obj) = response.as_object_mut() {
        obj.insert("cache".to_string(), json!({ "status": cache_status }));
        obj.insert(
            "engine".to_string(),
            json!({ "status": engine_status, "version": "why-engine-v1" }),
        );
        obj.insert(
            "generated_at".to_string(),
            json!(chrono::Utc::now().to_rfc3339()),
        );
    }
    response
}

async fn load_why_cache(db: &sqlx::PgPool, evidence_hash: &str) -> Option<Value> {
    let row: Result<Option<(Value,)>, sqlx::Error> = sqlx::query_as(
        "SELECT response FROM market.why_move_explanations WHERE evidence_hash = $1 AND expires_at > NOW()",
    )
    .bind(evidence_hash)
    .fetch_optional(db)
    .await;

    match row {
        Ok(Some((response,))) => Some(response),
        Ok(None) => None,
        Err(err) => {
            tracing::warn!(error = %err, "failed to load why-move cache");
            None
        }
    }
}

struct WhyCacheWrite<'a> {
    symbol: &'a str,
    window: &'a str,
    evidence_hash: &'a str,
    move_latest_at: Option<chrono::DateTime<chrono::Utc>>,
    move_pct: Option<f64>,
    response: &'a Value,
    evidence: &'a Value,
}

async fn store_why_cache(db: &sqlx::PgPool, entry: WhyCacheWrite<'_>) {
    let provider = entry
        .response
        .get("llm")
        .and_then(|llm| llm.get("provider"))
        .and_then(Value::as_str)
        .unwrap_or("deterministic");
    let model = entry
        .response
        .get("llm")
        .and_then(|llm| llm.get("model"))
        .and_then(Value::as_str);
    let status = entry
        .response
        .get("llm")
        .and_then(|llm| llm.get("status"))
        .and_then(Value::as_str)
        .unwrap_or("generated");
    let expires_at = chrono::Utc::now() + chrono::Duration::minutes(30);

    if let Err(err) = sqlx::query(
        "INSERT INTO market.why_move_explanations \
         (symbol, time_window, evidence_hash, move_latest_at, move_pct, engine_version, provider, model, status, response, evidence, expires_at) \
         VALUES ($1, $2, $3, $4, $5, 'why-engine-v1', $6, $7, $8, $9, $10, $11) \
         ON CONFLICT (evidence_hash) DO UPDATE SET response = EXCLUDED.response, evidence = EXCLUDED.evidence, status = EXCLUDED.status, expires_at = EXCLUDED.expires_at",
    )
    .bind(entry.symbol)
    .bind(entry.window)
    .bind(entry.evidence_hash)
    .bind(entry.move_latest_at)
    .bind(entry.move_pct)
    .bind(provider)
    .bind(model)
    .bind(status)
    .bind(entry.response)
    .bind(entry.evidence)
    .bind(expires_at)
    .execute(db)
    .await
    {
        tracing::warn!(error = %err, symbol = %entry.symbol, "failed to store why-move cache");
    }
}

async fn call_why_analyzer(evidence: &Value) -> anyhow::Result<Value> {
    let base =
        std::env::var("AI_SERVICE_URL").unwrap_or_else(|_| "http://localhost:5000".to_string());
    let url = format!("{}/why-did-it-move", base.trim_end_matches('/'));
    let res = HTTP_CLIENT
        .post(url)
        .json(evidence)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await?;
    let status = res.status();
    let text = res.text().await?;
    if !status.is_success() {
        anyhow::bail!("why analyzer HTTP error {status}: {text}");
    }
    Ok(serde_json::from_str(&text)?)
}

pub async fn why_did_it_move(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
    Query(query): Query<WhyQuery>,
) -> Json<Value> {
    let symbol = normalize_market_symbol(&symbol);
    if symbol.is_empty() {
        return Json(json!({ "error": "symbol is required" }));
    }

    let window_minutes = spike_window_minutes(query.window.as_deref());
    let window = format!("{window_minutes}m");
    let lookback_minutes = query.lookback_minutes.unwrap_or(180).clamp(30, 1440);
    let terms = symbol_terms(&symbol);

    let spike_rows = if let Some(clickhouse) = &state.clickhouse {
        match clickhouse.spike_candidates(window_minutes).await {
            Ok(rows) => rows,
            Err(err) => {
                tracing::warn!(error = %err, symbol = %symbol, "failed to load why-did-it-move spike context");
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };
    let candidate = spike_rows
        .iter()
        .find(|candidate| candidate.symbol.eq_ignore_ascii_case(&symbol));

    let latest_at = candidate
        .as_ref()
        .and_then(|candidate| chrono::DateTime::parse_from_rfc3339(&candidate.latest_at).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(chrono::Utc::now);
    let direction = candidate
        .as_ref()
        .map(|candidate| {
            if candidate.move_pct >= 0.0 {
                "up"
            } else {
                "down"
            }
        })
        .unwrap_or("none");
    let since = latest_at - chrono::Duration::minutes(lookback_minutes as i64);
    let until = latest_at + chrono::Duration::minutes(30);

    let news = match load_news_causes(&state.db, since, until).await {
        Ok(rows) => rows,
        Err(err) => {
            tracing::warn!(error = %err, symbol = %symbol, "failed to load why-did-it-move news causes");
            Vec::new()
        }
    };

    let mut scored: Vec<(f64, Vec<String>, NewsCause)> = news
        .into_iter()
        .filter_map(|cause| {
            let (score, matches) = score_cause(&cause, &terms, direction, latest_at);
            (score > 0.0).then_some((score, matches, cause))
        })
        .collect();
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(10);

    let top_score = scored.first().map(|row| row.0).unwrap_or(0.0);
    let confidence = confidence_for(top_score, scored.len());
    let matched: Vec<String> = scored
        .iter()
        .flat_map(|(_, terms, _)| terms.iter().cloned())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    let cross_assets: Vec<Value> = spike_rows
        .iter()
        .filter(|row| !row.symbol.eq_ignore_ascii_case(&symbol))
        .take(8)
        .map(|row| {
            json!({
                "symbol": row.symbol,
                "asset_type": row.asset_type,
                "move_pct": row.move_pct,
                "direction": if row.move_pct >= 0.0 { "up" } else { "down" },
                "latest_price": row.latest_price,
                "tick_count": row.tick_count,
                "latest_at": row.latest_at,
                "relationship": cross_asset_relationship(&symbol, &row.symbol, row.move_pct),
            })
        })
        .collect();
    let drivers = symbol_driver_terms(&symbol);
    let causes: Vec<Value> = scored
        .into_iter()
        .map(|(score, matches, cause)| {
            json!({
                "kind": cause.kind,
                "title": cause.title,
                "summary": cause.summary,
                "source_name": cause.source_name,
                "url": cause.url,
                "published_at": cause.published_at,
                "processed_at": cause.processed_at,
                "sentiment": cause.sentiment,
                "impact_level": cause.impact_level,
                "matched_terms": matches,
                "score": (score * 10.0).round() / 10.0,
                "reason": "Matched symbol context near the market move",
            })
        })
        .collect();

    let threshold =
        candidate.map(|candidate| spike_threshold(&candidate.symbol, &candidate.asset_type));
    let move_json = candidate.map(|candidate| {
        json!({
            "latest_price": candidate.latest_price,
            "baseline_price": candidate.baseline_price,
            "move_pct": candidate.move_pct,
            "direction": direction,
            "severity": threshold.map(|value| spike_severity(candidate.move_pct, value)),
            "threshold_pct": threshold,
            "tick_count": candidate.tick_count,
            "latest_at": candidate.latest_at,
            "is_active_spike": threshold.is_some_and(|value| candidate.move_pct.abs() >= value),
        })
    });
    let summary = why_summary(&symbol, candidate, causes.len(), confidence);
    let evidence = json!({
        "symbol": symbol,
        "window": window,
        "lookback_minutes": lookback_minutes,
        "move": move_json,
        "summary": summary,
        "confidence": confidence,
        "matched_terms": matched,
        "drivers": drivers,
        "cross_assets": cross_assets,
        "causes": {
            "news": causes,
            "calendar": [],
        },
    });
    let evidence_hash = evidence_hash(&evidence);
    if !query.refresh.unwrap_or(false) {
        if let Some(cached) = load_why_cache(&state.db, &evidence_hash).await {
            return Json(with_cache_metadata(cached, "hit", "cache"));
        }
    }

    match call_why_analyzer(&evidence).await {
        Ok(response) => {
            let response = with_cache_metadata(response, "miss", "analyzer");
            store_why_cache(
                &state.db,
                WhyCacheWrite {
                    symbol: &symbol,
                    window: &window,
                    evidence_hash: &evidence_hash,
                    move_latest_at: candidate
                        .and_then(|candidate| {
                            chrono::DateTime::parse_from_rfc3339(&candidate.latest_at).ok()
                        })
                        .map(|dt| dt.with_timezone(&chrono::Utc)),
                    move_pct: candidate.map(|candidate| candidate.move_pct),
                    response: &response,
                    evidence: &evidence,
                },
            )
            .await;
            Json(response)
        }
        Err(err) => {
            tracing::warn!(error = %err, symbol = %symbol, "why analyzer unavailable, using fallback response");
            Json(json!({
                "symbol": evidence["symbol"].clone(),
                "window": evidence["window"].clone(),
                "lookback_minutes": lookback_minutes,
                "move": evidence["move"].clone(),
                "summary": summary,
                "headline": format!("{} move context", symbol),
                "explanation": summary,
                "confidence": { "label": confidence, "score": top_score, "breakdown": {} },
                "matched_terms": evidence["matched_terms"].clone(),
                "drivers": evidence["drivers"].clone(),
                "cross_assets": evidence["cross_assets"].clone(),
                "causes": evidence["causes"].clone(),
                "llm": { "provider": "gemini", "model": null, "status": "fallback", "narrative": null },
                "engine": { "status": "fallback", "version": "rust-fallback-v1" },
                "cache": { "status": "bypass", "evidence_hash": evidence_hash },
                "evidence": evidence,
                "generated_at": chrono::Utc::now().to_rfc3339(),
            }))
        }
    }
}

pub async fn data_quality(State(state): State<AppState>) -> Json<Value> {
    let latest_prices = match load_latest_prices(&state.db).await {
        Ok(prices) => prices,
        Err(err) => {
            tracing::warn!(error = %err, "failed to load latest prices for data quality");
            Vec::new()
        }
    };

    let tick_stats = if let Some(clickhouse) = &state.clickhouse {
        match clickhouse.tick_stats().await {
            Ok(rows) => rows,
            Err(err) => {
                tracing::warn!(error = %err, "failed to load ClickHouse tick stats for data quality");
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };

    let stats_by_symbol: std::collections::HashMap<String, Value> = tick_stats
        .into_iter()
        .filter_map(|row| {
            let symbol = row.get("symbol").and_then(|v| v.as_str())?.to_string();
            Some((symbol, row))
        })
        .collect();
    let now = chrono::Utc::now().timestamp();
    let items: Vec<Value> = latest_prices
        .into_iter()
        .map(|price| {
            let stats = stats_by_symbol.get(&price.symbol);
            let last_tick_time = stats
                .and_then(|row| row.get("last_tick_time"))
                .and_then(|v| v.as_i64());
            let latest_received_at = price
                .received_at
                .as_deref()
                .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
                .map(|dt| dt.timestamp());
            let last_seen = last_tick_time.or(latest_received_at);
            let age_sec = last_seen.map(|ts| (now - ts).max(0));
            let ticks_5m = stats
                .and_then(|row| row.get("ticks_5m"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let unique_prices_1h = stats
                .and_then(|row| row.get("unique_prices_1h"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let status = if age_sec.is_none() {
                "unknown"
            } else if age_sec.is_some_and(|age| age > 15 * 60) {
                "stale"
            } else if ticks_5m == 0 {
                "quiet"
            } else if unique_prices_1h <= 1 && !price.asset_type.eq_ignore_ascii_case("crypto") {
                "flat"
            } else {
                "ok"
            };

            json!({
                "symbol": price.symbol,
                "asset_type": price.asset_type,
                "latest_price": price.price,
                "source": price.source,
                "received_at": price.received_at,
                "age_sec": age_sec,
                "ticks_5m": ticks_5m,
                "ticks_1h": stats.and_then(|row| row.get("ticks_1h")).and_then(|v| v.as_u64()).unwrap_or(0),
                "unique_prices_1h": unique_prices_1h,
                "status": status,
            })
        })
        .collect();

    Json(json!({
        "items": items,
        "total": items.len(),
        "generated_at": chrono::Utc::now().to_rfc3339(),
    }))
}

#[derive(Deserialize)]
pub struct HistoryQuery {
    pub resolution: Option<String>,
}

fn normalize_history_resolution(resolution: Option<&str>) -> &'static str {
    match resolution.unwrap_or("1m") {
        "5m" => "5m",
        "15m" => "15m",
        "1h" => "1h",
        _ => "1m",
    }
}

pub async fn get_history(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
    Query(query): Query<HistoryQuery>,
) -> Json<Value> {
    let sym = symbol.to_uppercase();
    let resolution = normalize_history_resolution(query.resolution.as_deref());

    if let Some(clickhouse) = &state.clickhouse {
        match clickhouse.latest_history(&sym, resolution, 120).await {
            Ok(history) if history_is_usable(&history, chrono::Utc::now()) => {
                return Json(json!(history));
            }
            Ok(history) if !history.is_empty() => {
                tracing::warn!(symbol = %sym, points = history.len(), "ClickHouse market history is flat or stale, falling back");
            }
            Ok(_) => {}
            Err(err) => {
                tracing::warn!(error = %err, symbol = %sym, "failed to query ClickHouse market history");
            }
        }
    }

    let db_res: Result<Vec<(chrono::DateTime<chrono::Utc>, f64)>, _> = sqlx::query_as(
        "SELECT time, close FROM market.ohlcv_candles \
         WHERE symbol = $1 AND resolution = '1m' \
         ORDER BY time DESC LIMIT 120",
    )
    .bind(&sym)
    .fetch_all(&state.db)
    .await;

    if let Ok(candles) = db_res {
        if !candles.is_empty() {
            let mut history: Vec<Value> = candles
                .into_iter()
                .map(|(t, close)| {
                    json!({
                        "time": t.timestamp(),
                        "value": close
                    })
                })
                .collect();
            history.reverse();
            if history_is_usable(&history, chrono::Utc::now()) {
                return Json(json!(history));
            }
            tracing::warn!(symbol = %sym, points = history.len(), "market history DB candles are flat or stale, falling back to reference history");
        }
    }

    if sym.ends_with("USDT") {
        let Ok(template) = std::env::var("CRYPTO_HISTORY_URL_TEMPLATE") else {
            return Json(json!(latest_price_history_fallback(&state.db, &sym).await));
        };
        let url = template.replace("{symbol}", &sym);
        match HTTP_CLIENT.get(&url).send().await {
            Ok(res) => {
                let status = res.status();
                if !status.is_success() {
                    tracing::warn!("crypto history HTTP error for {}: {}", sym, status);
                    return Json(json!(latest_price_history_fallback(&state.db, &sym).await));
                }
                match res.json::<Vec<Vec<Value>>>().await {
                    Ok(data) => {
                        let history: Vec<Value> = data
                            .iter()
                            .filter_map(|item| {
                                if item.len() >= 5 {
                                    let time_ms = item[0].as_i64()?;
                                    let close_str = item[4].as_str()?;
                                    let close_val: f64 = close_str.parse().ok()?;
                                    Some(json!({
                                        "time": time_ms / 1000,
                                        "value": close_val
                                    }))
                                } else {
                                    None
                                }
                            })
                            .collect();
                        return Json(json!(history));
                    }
                    Err(err) => {
                        tracing::warn!(
                            "Failed to parse crypto history JSON for {}: {:?}",
                            sym,
                            err
                        );
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to fetch crypto history for {}: {:?}", sym, e);
            }
        }
    }

    let reference_symbol = tradingview_symbol(&sym);
    let template = std::env::var("TRADINGVIEW_HISTORY_URL_TEMPLATE")
        .or_else(|_| std::env::var("REFERENCE_HISTORY_URL_TEMPLATE"))
        .unwrap_or_default();
    if template.trim().is_empty() {
        tracing::warn!(symbol = %sym, ticker = %reference_symbol, "reference history URL template not configured, using last known price fallback");
        return Json(json!(latest_price_history_fallback(&state.db, &sym).await));
    }
    let url = template
        .trim()
        .replace("{symbol}", &encode_symbol(&reference_symbol));

    match HTTP_CLIENT.get(&url).send().await {
        Ok(res) => {
            let status = res.status();
            if !status.is_success() {
                tracing::warn!(
                    "reference history HTTP error for {} (ticker: {}): {}",
                    sym,
                    reference_symbol,
                    status
                );
                return Json(json!(latest_price_history_fallback(&state.db, &sym).await));
            }
            match res.json::<Value>().await {
                Ok(json_data) => {
                    let history = parse_reference_history(&json_data);
                    if !history.is_empty() {
                        return Json(json!(history));
                    }
                    tracing::warn!(
                        "reference history JSON structure mismatch for {} (ticker: {})",
                        sym,
                        reference_symbol
                    );
                }
                Err(err) => {
                    tracing::warn!(
                        "Failed to parse reference history JSON for {} (ticker: {}): {:?}",
                        sym,
                        reference_symbol,
                        err
                    );
                }
            }
        }
        Err(e) => {
            tracing::warn!(
                "Failed to fetch reference history for {} (ticker: {}): {:?}",
                sym,
                reference_symbol,
                e
            );
        }
    }

    Json(json!(latest_price_history_fallback(&state.db, &sym).await))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn maps_symbols_to_tradingview() {
        assert_eq!(tradingview_symbol("XAUUSD"), "OANDA:XAUUSD");
        assert_eq!(tradingview_symbol("SPX"), "SP:SPX");
        assert_eq!(tradingview_symbol("DXY"), "TVC:DXY");
        assert_eq!(tradingview_symbol("EURUSD"), "FX:EURUSD");
    }

    #[test]
    fn parses_history_shapes() {
        let bars = parse_reference_history(&json!([
            { "time": 1_700_000_000, "close": 2000.5 },
            { "timestamp": 1_700_000_060_000i64, "value": "2001.5" }
        ]));
        assert_eq!(bars.len(), 2);
        assert_eq!(bars[1]["time"], 1_700_000_060);
        assert_eq!(bars[1]["value"], 2001.5);

        let compact = parse_reference_history(&json!({
            "t": [1_700_000_000, 1_700_000_060],
            "c": [1.08, 1.09]
        }));
        assert_eq!(compact.len(), 2);
        assert_eq!(compact[0]["value"], 1.08);
    }

    #[test]
    fn rejects_flat_or_stale_history() {
        let now = chrono::Utc::now();
        let start = now.timestamp() - 4 * 60;
        let flat: Vec<Value> = (0..5)
            .map(|i| json!({ "time": start + i * 60, "value": 100.0 }))
            .collect();
        let moving: Vec<Value> = (0..5)
            .map(|i| json!({ "time": start + i * 60, "value": 100.0 + i as f64 * 0.1 }))
            .collect();
        let stale: Vec<Value> = (0..5)
            .map(|i| json!({ "time": start - 7 * 60 * 60 + i * 60, "value": 100.0 + i as f64 * 0.1 }))
            .collect();

        assert!(!history_is_usable(&flat, now));
        assert!(history_is_usable(&moving, now));
        assert!(!history_is_usable(&stale, now));
    }

    #[test]
    fn normalizes_history_resolution() {
        assert_eq!(normalize_history_resolution(None), "1m");
        assert_eq!(normalize_history_resolution(Some("1m")), "1m");
        assert_eq!(normalize_history_resolution(Some("5m")), "5m");
        assert_eq!(normalize_history_resolution(Some("15m")), "15m");
        assert_eq!(normalize_history_resolution(Some("1h")), "1h");
        assert_eq!(normalize_history_resolution(Some("1d")), "1m");
    }

    #[test]
    fn classifies_spike_thresholds() {
        assert_eq!(spike_window_minutes(None), 5);
        assert_eq!(spike_window_minutes(Some("15m")), 15);
        assert_eq!(spike_window_minutes(Some("bad")), 5);
        assert_eq!(spike_threshold("DXY", "index"), 0.12);
        assert_eq!(spike_threshold("XAUUSD", "forex"), 0.25);
        assert_eq!(spike_threshold("BTCUSDT", "crypto"), 0.80);
        assert_eq!(spike_threshold("EURUSD", "forex"), 0.15);
        assert_eq!(spike_severity(0.24, 0.12), "high");
        assert_eq!(spike_severity(0.13, 0.12), "medium");
    }

    #[test]
    fn builds_symbol_terms_for_why_explanations() {
        assert_eq!(normalize_market_symbol(" xau/usd "), "XAUUSD");
        let gold_terms = symbol_terms("XAUUSD");
        assert!(gold_terms.contains(&"GOLD".to_string()));
        assert!(gold_terms.contains(&"USD".to_string()));

        let fx_terms = symbol_terms("eurusd");
        assert!(fx_terms.contains(&"EUR".to_string()));
        assert!(fx_terms.contains(&"USD".to_string()));
    }

    #[test]
    fn scores_relevant_aligned_news_higher() {
        let latest_at = chrono::Utc::now();
        let cause = NewsCause {
            kind: "forex_news",
            title: "Gold rallies as Fed inflation worries hit USD".to_string(),
            summary: Some("XAU jumps after yields fall".to_string()),
            source_name: Some("Test".to_string()),
            url: None,
            published_at: Some(latest_at - chrono::Duration::minutes(10)),
            processed_at: Some(latest_at - chrono::Duration::minutes(10)),
            sentiment: Some("positive".to_string()),
            impact_level: Some("high".to_string()),
            searchable: "Gold rallies as Fed inflation worries hit USD XAU jumps".to_string(),
        };

        let (score, matches) = score_cause(&cause, &symbol_terms("XAUUSD"), "up", latest_at);

        assert!(score >= 45.0);
        assert!(matches.contains(&"GOLD".to_string()));
        assert_eq!(confidence_for(score, 2), "high");
    }

    #[test]
    fn ignores_unrelated_news_for_why_explanations() {
        let latest_at = chrono::Utc::now();
        let cause = NewsCause {
            kind: "stock_news",
            title: "Unrelated earnings update".to_string(),
            summary: None,
            source_name: None,
            url: None,
            published_at: Some(latest_at),
            processed_at: Some(latest_at),
            sentiment: Some("neutral".to_string()),
            impact_level: None,
            searchable: "Unrelated earnings update".to_string(),
        };

        let (score, matches) = score_cause(&cause, &symbol_terms("XAUUSD"), "up", latest_at);

        assert_eq!(score, 0.0);
        assert!(matches.is_empty());
    }
}
