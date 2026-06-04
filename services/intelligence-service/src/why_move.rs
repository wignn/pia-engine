use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::clickhouse::SpikeCandidate;
use crate::state::AppState;

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

type MacroSignalRow = (
    String,
    String,
    String,
    chrono::NaiveDate,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    String,
    String,
    String,
);

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
                tracing::warn!(error = %err, symbol = %symbol, "failed to load why-move spike context");
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
        .and_then(|candidate| chrono::DateTime::parse_from_rfc3339(&candidate.latest_at).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(chrono::Utc::now);
    let direction = candidate
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

    let news = load_news_causes(&state.db, since, until)
        .await
        .unwrap_or_else(|err| {
            tracing::warn!(error = %err, symbol = %symbol, "failed to load why-move news causes");
            Vec::new()
        });

    let macro_signals = load_macro_signals(&state.db).await.unwrap_or_else(|err| {
        tracing::warn!(error = %err, symbol = %symbol, "failed to load macro signals");
        Vec::new()
    });

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
        "causes": { "news": causes, "calendar": [], "macro": macro_signals },
    });
    let evidence_hash = evidence_hash(&evidence);
    if !query.refresh.unwrap_or(false) {
        if let Some(cached) = load_why_cache(&state.db, &evidence_hash).await {
            return Json(with_cache_metadata(cached, "hit", "cache"));
        }
    }

    match call_why_analyzer(&state, &evidence).await {
        Ok(response) => {
            let response = preserve_canonical_context(response, &evidence);
            let response = with_cache_metadata(response, "miss", "analyzer");
            store_why_cache(
                &state.db,
                &symbol,
                &window,
                &evidence_hash,
                candidate,
                &response,
                &evidence,
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
        _ if symbol.ends_with("USDT") => {
            terms.extend(["CRYPTO", "KRIPTO", "USDT"].map(str::to_string));
            terms.push(symbol.trim_end_matches("USDT").to_string());
        }
        _ if symbol.len() == 6 => {
            terms.push(symbol[0..3].to_string());
            terms.push(symbol[3..6].to_string());
        }
        _ => {}
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
    match (
        normalize_market_symbol(symbol).as_str(),
        normalize_market_symbol(other_symbol).as_str(),
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
        Some(candidate) if cause_count > 0 => format!("{symbol} moved {} {:.2}% over the selected window with {cause_count} relevant news catalyst(s) nearby. Confidence is {confidence}.", if candidate.move_pct >= 0.0 { "up" } else { "down" }, candidate.move_pct.abs()),
        Some(candidate) => format!("{symbol} moved {} {:.2}% over the selected window, but no matching news catalyst was found in the lookback window.", if candidate.move_pct >= 0.0 { "up" } else { "down" }, candidate.move_pct.abs()),
        None => format!("No recent market move context is available for {symbol}."),
    }
}

async fn load_news_causes(
    db: &sqlx::PgPool,
    since: chrono::DateTime<chrono::Utc>,
    until: chrono::DateTime<chrono::Utc>,
) -> Result<Vec<NewsCause>, sqlx::Error> {
    let forex_rows: Vec<ForexCauseRow> = sqlx::query_as("SELECT a.original_title, a.summary, COALESCE(s.name, 'Unknown') AS source_name, a.original_url, a.published_at, a.processed_at, an.sentiment, an.impact_level, COALESCE(an.currency_pairs, '') FROM news.forex_news_articles a LEFT JOIN news.forex_news_sources s ON a.source_id = s.id LEFT JOIN news.forex_news_analyses an ON a.id = an.article_id WHERE a.is_processed = TRUE AND COALESCE(a.processed_at, a.published_at, a.created_at) BETWEEN $1 AND $2 ORDER BY COALESCE(a.processed_at, a.published_at, a.created_at) DESC LIMIT 100").bind(since).bind(until).fetch_all(db).await?;
    let stock_rows: Vec<StockCauseRow> = sqlx::query_as("SELECT title, summary, source_name, tickers, sentiment, impact_level, processed_at FROM news.stock_news WHERE is_processed = TRUE AND COALESCE(processed_at, created_at) BETWEEN $1 AND $2 ORDER BY COALESCE(processed_at, created_at) DESC LIMIT 100").bind(since).bind(until).fetch_all(db).await?;
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

async fn load_macro_signals(db: &sqlx::PgPool) -> Result<Vec<Value>, sqlx::Error> {
    let rows: Vec<MacroSignalRow> = sqlx::query_as(
        "WITH latest AS (
            SELECT DISTINCT ON (s.series_id) s.series_id, ms.title, s.category, s.signal_date, s.latest_value, s.change_1d, s.change_7d, s.direction, s.severity, s.narrative
            FROM news.macro_signals s
            JOIN news.macro_series ms ON ms.id = s.series_id
            WHERE s.severity IN ('high', 'medium')
            ORDER BY s.series_id, s.signal_date DESC
        )
        SELECT * FROM latest
        ORDER BY CASE severity WHEN 'high' THEN 1 ELSE 2 END, category ASC, series_id ASC
        LIMIT 12",
    )
    .fetch_all(db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            json!({
                "series_id": row.0,
                "title": row.1,
                "category": row.2,
                "signal_date": row.3,
                "latest_value": row.4,
                "change_1d": row.5,
                "change_7d": row.6,
                "direction": row.7,
                "severity": row.8,
                "narrative": row.9,
            })
        })
        .collect())
}

fn evidence_hash(evidence: &Value) -> String {
    let serialized = serde_json::to_string(evidence).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(serialized.as_bytes());
    hex::encode(hasher.finalize())
}

fn preserve_canonical_context(mut response: Value, evidence: &Value) -> Value {
    if let Some(obj) = response.as_object_mut() {
        if obj.get("move").is_none_or(Value::is_null) {
            obj.insert("move".to_string(), evidence["move"].clone());
        }
        if obj.get("evidence").is_none_or(Value::is_null) {
            obj.insert("evidence".to_string(), evidence.clone());
        }
    }
    response
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
    sqlx::query_as::<_, (Value,)>("SELECT response FROM market.why_move_explanations WHERE evidence_hash = $1 AND expires_at > NOW()").bind(evidence_hash).fetch_optional(db).await.ok().flatten().map(|row| row.0)
}

async fn store_why_cache(
    db: &sqlx::PgPool,
    symbol: &str,
    window: &str,
    evidence_hash: &str,
    candidate: Option<&SpikeCandidate>,
    response: &Value,
    evidence: &Value,
) {
    let expires_at = chrono::Utc::now() + chrono::Duration::minutes(30);
    let move_latest_at = candidate
        .and_then(|candidate| chrono::DateTime::parse_from_rfc3339(&candidate.latest_at).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));
    let move_pct = candidate.map(|candidate| candidate.move_pct);
    let _ = sqlx::query("INSERT INTO market.why_move_explanations (symbol, time_window, evidence_hash, move_latest_at, move_pct, engine_version, provider, model, status, response, evidence, expires_at) VALUES ($1, $2, $3, $4, $5, 'why-engine-v1', 'deterministic', NULL, 'generated', $6, $7, $8) ON CONFLICT (evidence_hash) DO UPDATE SET response = EXCLUDED.response, evidence = EXCLUDED.evidence, status = EXCLUDED.status, expires_at = EXCLUDED.expires_at")
        .bind(symbol).bind(window).bind(evidence_hash).bind(move_latest_at).bind(move_pct).bind(response).bind(evidence).bind(expires_at).execute(db).await;
}

async fn call_why_analyzer(state: &AppState, evidence: &Value) -> anyhow::Result<Value> {
    let url = format!(
        "{}/why-did-it-move",
        state.config.ai_service_url.trim_end_matches('/')
    );
    let res = state
        .http
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

fn spike_window_minutes(window: Option<&str>) -> u32 {
    match window.unwrap_or("5m") {
        "1m" => 1,
        "15m" => 15,
        "30m" => 30,
        "1h" => 60,
        _ => 5,
    }
}

fn spike_threshold(symbol: &str, asset_type: &str) -> f64 {
    if asset_type.eq_ignore_ascii_case("crypto") || symbol.ends_with("USDT") {
        0.5
    } else {
        0.1
    }
}

fn spike_severity(move_pct: f64, threshold: f64) -> &'static str {
    let ratio = move_pct.abs() / threshold.max(0.0001);
    if ratio >= 5.0 {
        "critical"
    } else if ratio >= 3.0 {
        "high"
    } else if ratio >= 1.5 {
        "medium"
    } else {
        "low"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symbol_terms_include_gold_context() {
        let terms = symbol_terms("xauusd");
        assert!(terms.contains(&"GOLD".to_string()));
        assert!(terms.contains(&"USD".to_string()));
    }

    #[test]
    fn confidence_uses_score_and_cause_count() {
        assert_eq!(confidence_for(50.0, 2), "high");
        assert_eq!(confidence_for(30.0, 1), "medium");
        assert_eq!(confidence_for(10.0, 3), "low");
    }

    #[test]
    fn analyzer_response_preserves_canonical_move_context() {
        let response = json!({
            "symbol": "XAUUSD",
            "move": null,
            "evidence": null,
            "llm": { "status": "generated", "narrative": { "headline": "Gold moved", "explanation": "Macro context", "drivers": [], "confidence": "medium", "caveats": [] } }
        });
        let evidence = json!({
            "symbol": "XAUUSD",
            "window": "5m",
            "move": {
                "latest_price": 4450.02,
                "baseline_price": 4449.0,
                "move_pct": 0.0229,
                "direction": "up",
                "tick_count": 15,
                "latest_at": "2026-05-27 14:19:54.305"
            },
            "causes": { "news": [], "calendar": [] },
            "cross_assets": []
        });

        let merged = preserve_canonical_context(response, &evidence);

        assert_eq!(merged["move"], evidence["move"]);
        assert_eq!(merged["evidence"], evidence);
    }
}
