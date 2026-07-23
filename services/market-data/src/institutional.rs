use axum::extract::{Query, State};
use axum::Json;
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use std::time::Duration;
use tracing::{error, info, warn};

use crate::config::Config;
use crate::state::AppState;

const DEFAULT_SYMBOLS: &str = "SPY,QQQ,AAPL,MSFT,TSLA,NVDA,GLD";
const NASDAQ_HALTS_URL: &str = "https://www.nasdaqtrader.com/dynamic/symdir/tradinghalts.txt";

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TradingHaltRow {
    pub source: String,
    pub symbol: String,
    pub issue_name: Option<String>,
    pub market: Option<String>,
    pub reason_code: Option<String>,
    pub halt_date: NaiveDate,
    pub halt_time: Option<NaiveTime>,
    pub resume_date: Option<NaiveDate>,
    pub resume_quote_time: Option<NaiveTime>,
    pub resume_trade_time: Option<NaiveTime>,
    pub raw_json: Option<Value>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CorporateActionRow {
    pub id: String,
    pub source: String,
    pub symbol: String,
    pub action_type: String,
    pub ex_date: NaiveDate,
    pub amount: Option<f64>,
    pub ratio: Option<String>,
    pub description: Option<String>,
    pub raw_json: Option<Value>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RealizedVolatilityRow {
    pub symbol: String,
    pub window_days: i32,
    pub date: NaiveDate,
    pub realized_volatility: f64,
    pub source: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ImpliedVolatilityRow {
    pub symbol: String,
    pub underlying_price: f64,
    pub iv_atm: Option<f64>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SymbolLimitQuery {
    pub symbol: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CorporateActionsQuery {
    pub symbol: Option<String>,
    pub action_type: Option<String>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct RealizedVolatilityQuery {
    pub symbol: Option<String>,
    pub window_days: Option<i32>,
    pub limit: Option<i64>,
}

pub async fn get_trading_halts(
    State(state): State<AppState>,
    Query(query): Query<SymbolLimitQuery>,
) -> Json<Value> {
    let limit = query.limit.unwrap_or(100).clamp(1, 500);
    match query_trading_halts(&state.db, query.symbol.as_deref(), limit).await {
        Ok(rows) => Json(serde_json::json!({ "data": rows })),
        Err(err) => {
            error!(error = %err, "failed to query trading halts");
            Json(serde_json::json!({ "error": "internal server error" }))
        }
    }
}

pub async fn get_corporate_actions(
    State(state): State<AppState>,
    Query(query): Query<CorporateActionsQuery>,
) -> Json<Value> {
    let limit = query.limit.unwrap_or(100).clamp(1, 500);
    match query_corporate_actions(
        &state.db,
        query.symbol.as_deref(),
        query.action_type.as_deref(),
        query.from,
        query.to,
        limit,
    )
    .await
    {
        Ok(rows) => Json(serde_json::json!({ "data": rows })),
        Err(err) => {
            error!(error = %err, "failed to query corporate actions");
            Json(serde_json::json!({ "error": "internal server error" }))
        }
    }
}

pub async fn get_realized_volatility(
    State(state): State<AppState>,
    Query(query): Query<RealizedVolatilityQuery>,
) -> Json<Value> {
    let limit = query.limit.unwrap_or(100).clamp(1, 500);
    match query_realized_volatility(&state.db, query.symbol.as_deref(), query.window_days, limit)
        .await
    {
        Ok(rows) => Json(serde_json::json!({ "data": rows })),
        Err(err) => {
            error!(error = %err, "failed to query realized volatility");
            Json(serde_json::json!({ "error": "internal server error" }))
        }
    }
}

pub async fn get_implied_volatility(
    State(state): State<AppState>,
    Query(query): Query<SymbolLimitQuery>,
) -> Json<Value> {
    let limit = query.limit.unwrap_or(100).clamp(1, 500);
    match query_implied_volatility(&state.db, query.symbol.as_deref(), limit).await {
        Ok(rows) => Json(serde_json::json!({ "data": rows })),
        Err(err) => {
            error!(error = %err, "failed to query implied volatility");
            Json(serde_json::json!({ "error": "internal server error" }))
        }
    }
}

pub async fn run_sync(config: Config, pool: PgPool) {
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("atlsd-market-data/1.0")
        .build()
        .unwrap_or_default();

    loop {
        sync_all(&pool, &http, &config.institutional_symbols).await;
        tokio::time::sleep(Duration::from_secs(config.institutional_sync_sec)).await;
    }
}

async fn sync_all(pool: &PgPool, http: &reqwest::Client, symbols_csv: &str) {
    info!("starting institutional data sync");
    if let Err(err) = sync_trading_halts(pool, http).await {
        warn!(error = %err, "trading halt sync failed");
    }

    for symbol in symbols(symbols_csv) {
        if let Err(err) = sync_corporate_actions(pool, http, &symbol).await {
            warn!(error = %err, symbol = %symbol, "corporate action sync failed");
        }
    }

    for window in [20, 60] {
        if let Err(err) = sync_realized_volatility(pool, window).await {
            warn!(error = %err, window_days = window, "realized volatility sync failed");
        }
    }
}

async fn query_trading_halts(
    pool: &PgPool,
    symbol: Option<&str>,
    limit: i64,
) -> Result<Vec<TradingHaltRow>, sqlx::Error> {
    sqlx::query_as::<_, TradingHaltRow>(
        r#"
        SELECT source, symbol, issue_name, market, reason_code, halt_date, halt_time,
               resume_date, resume_quote_time, resume_trade_time, raw_json, updated_at
        FROM trading_halts
        WHERE ($1::text IS NULL OR UPPER(symbol) = UPPER($1))
        ORDER BY halt_date DESC, halt_time DESC NULLS LAST
        LIMIT $2
        "#,
    )
    .bind(symbol)
    .bind(limit)
    .fetch_all(pool)
    .await
}

async fn query_corporate_actions(
    pool: &PgPool,
    symbol: Option<&str>,
    action_type: Option<&str>,
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
    limit: i64,
) -> Result<Vec<CorporateActionRow>, sqlx::Error> {
    sqlx::query_as::<_, CorporateActionRow>(
        r#"
        SELECT id, source, symbol, action_type, ex_date, amount, ratio, description, raw_json, updated_at
        FROM corporate_actions
        WHERE ($1::text IS NULL OR UPPER(symbol) = UPPER($1))
          AND ($2::text IS NULL OR action_type = $2)
          AND ($3::date IS NULL OR ex_date >= $3)
          AND ($4::date IS NULL OR ex_date <= $4)
        ORDER BY ex_date DESC
        LIMIT $5
        "#,
    )
    .bind(symbol)
    .bind(action_type)
    .bind(from)
    .bind(to)
    .bind(limit)
    .fetch_all(pool)
    .await
}

async fn query_realized_volatility(
    pool: &PgPool,
    symbol: Option<&str>,
    window_days: Option<i32>,
    limit: i64,
) -> Result<Vec<RealizedVolatilityRow>, sqlx::Error> {
    sqlx::query_as::<_, RealizedVolatilityRow>(
        r#"
        SELECT symbol, window_days, date, realized_volatility, source, updated_at
        FROM realized_volatility
        WHERE ($1::text IS NULL OR UPPER(symbol) = UPPER($1))
          AND ($2::int IS NULL OR window_days = $2)
        ORDER BY date DESC, symbol ASC, window_days ASC
        LIMIT $3
        "#,
    )
    .bind(symbol)
    .bind(window_days)
    .bind(limit)
    .fetch_all(pool)
    .await
}

async fn query_implied_volatility(
    pool: &PgPool,
    symbol: Option<&str>,
    limit: i64,
) -> Result<Vec<ImpliedVolatilityRow>, sqlx::Error> {
    sqlx::query_as::<_, ImpliedVolatilityRow>(
        r#"
        SELECT symbol, underlying_price, iv_atm, updated_at
        FROM options_snapshots
        WHERE ($1::text IS NULL OR UPPER(symbol) = UPPER($1))
        ORDER BY updated_at DESC, symbol ASC
        LIMIT $2
        "#,
    )
    .bind(symbol)
    .bind(limit)
    .fetch_all(pool)
    .await
}

async fn sync_trading_halts(pool: &PgPool, http: &reqwest::Client) -> anyhow::Result<()> {
    let body = http.get(NASDAQ_HALTS_URL).send().await?.text().await?;
    for halt in parse_nasdaq_halts(&body) {
        sqlx::query(
            r#"
            INSERT INTO trading_halts (
                source, symbol, halt_date, halt_time, issue_name, market, reason_code,
                resume_date, resume_quote_time, resume_trade_time, raw_json, updated_at
            ) VALUES ('nasdaq_trader', $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW())
            ON CONFLICT (source, symbol, halt_date, halt_time) DO UPDATE SET
                issue_name = EXCLUDED.issue_name,
                market = EXCLUDED.market,
                reason_code = EXCLUDED.reason_code,
                resume_date = EXCLUDED.resume_date,
                resume_quote_time = EXCLUDED.resume_quote_time,
                resume_trade_time = EXCLUDED.resume_trade_time,
                raw_json = EXCLUDED.raw_json,
                updated_at = NOW()
            "#,
        )
        .bind(&halt.symbol)
        .bind(halt.halt_date)
        .bind(halt.halt_time)
        .bind(&halt.issue_name)
        .bind(&halt.market)
        .bind(&halt.reason_code)
        .bind(halt.resume_date)
        .bind(halt.resume_quote_time)
        .bind(halt.resume_trade_time)
        .bind(&halt.raw_json)
        .execute(pool)
        .await?;
    }
    Ok(())
}

async fn sync_corporate_actions(
    pool: &PgPool,
    http: &reqwest::Client,
    symbol: &str,
) -> anyhow::Result<()> {
    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{symbol}?range=2y&interval=1d&events=div%2Csplits"
    );
    let root: Value = http.get(url).send().await?.json().await?;
    let events = root
        .pointer("/chart/result/0/events")
        .unwrap_or(&Value::Null);

    if let Some(dividends) = events.get("dividends").and_then(|v| v.as_object()) {
        for item in dividends.values() {
            let Some(date) = unix_date(item.get("date").and_then(|v| v.as_i64())) else {
                continue;
            };
            let amount = item.get("amount").and_then(|v| v.as_f64());
            upsert_corporate_action(
                pool,
                &format!("yahoo:{symbol}:dividend:{date}"),
                symbol,
                "dividend",
                date,
                amount,
                None,
                amount.map(|v| format!("Dividend {v}")),
                item,
            )
            .await?;
        }
    }

    if let Some(splits) = events.get("splits").and_then(|v| v.as_object()) {
        for item in splits.values() {
            let Some(date) = unix_date(item.get("date").and_then(|v| v.as_i64())) else {
                continue;
            };
            let numerator = item
                .get("numerator")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let denominator = item
                .get("denominator")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let ratio = item
                .get("splitRatio")
                .and_then(|v| v.as_str())
                .map(str::to_string)
                .or_else(|| Some(format_ratio(numerator, denominator)));
            upsert_corporate_action(
                pool,
                &format!("yahoo:{symbol}:split:{date}"),
                symbol,
                "split",
                date,
                None,
                ratio.clone(),
                ratio.map(|v| format!("Split {v}")),
                item,
            )
            .await?;
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn upsert_corporate_action(
    pool: &PgPool,
    id: &str,
    symbol: &str,
    action_type: &str,
    ex_date: NaiveDate,
    amount: Option<f64>,
    ratio: Option<String>,
    description: Option<String>,
    raw_json: &Value,
) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO corporate_actions (id, source, symbol, action_type, ex_date, amount, ratio, description, raw_json, updated_at)
        VALUES ($1, 'yahoo_chart', $2, $3, $4, $5, $6, $7, $8, NOW())
        ON CONFLICT (id) DO UPDATE SET
            amount = EXCLUDED.amount,
            ratio = EXCLUDED.ratio,
            description = EXCLUDED.description,
            raw_json = EXCLUDED.raw_json,
            updated_at = NOW()
        "#,
    )
    .bind(id)
    .bind(symbol)
    .bind(action_type)
    .bind(ex_date)
    .bind(amount)
    .bind(ratio)
    .bind(description)
    .bind(raw_json)
    .execute(pool)
    .await?;
    Ok(())
}

async fn sync_realized_volatility(pool: &PgPool, window_days: i32) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO realized_volatility (symbol, window_days, date, realized_volatility, source, updated_at)
        WITH daily AS (
            SELECT symbol, time::date AS date, (array_agg(close ORDER BY time DESC))[1] AS close
            FROM market.ohlcv_candles
            WHERE time >= NOW() - (($1::int + 10) * INTERVAL '1 day')
              AND close > 0
            GROUP BY symbol, time::date
        ), returns AS (
            SELECT symbol, date, ln(close / LAG(close) OVER (PARTITION BY symbol ORDER BY date)) AS ret
            FROM daily
        ), calc AS (
            SELECT symbol, MAX(date) AS date, STDDEV_SAMP(ret) * SQRT(252.0) AS realized_volatility
            FROM returns
            WHERE ret IS NOT NULL
            GROUP BY symbol
            HAVING COUNT(ret) >= 2
        )
        SELECT symbol, $1, date, realized_volatility, 'ohlcv_candles', NOW()
        FROM calc
        WHERE realized_volatility IS NOT NULL
        ON CONFLICT (symbol, window_days, date) DO UPDATE SET
            realized_volatility = EXCLUDED.realized_volatility,
            source = EXCLUDED.source,
            updated_at = NOW()
        "#,
    )
    .bind(window_days)
    .execute(pool)
    .await?;
    Ok(())
}

fn symbols(csv: &str) -> Vec<String> {
    let raw = if csv.trim().is_empty() {
        DEFAULT_SYMBOLS
    } else {
        csv
    };
    raw.split(',')
        .map(|s| s.trim().to_uppercase())
        .filter(|s| !s.is_empty())
        .collect()
}

#[derive(Debug)]
struct NasdaqHalt {
    symbol: String,
    issue_name: Option<String>,
    market: Option<String>,
    reason_code: Option<String>,
    halt_date: NaiveDate,
    halt_time: Option<NaiveTime>,
    resume_date: Option<NaiveDate>,
    resume_quote_time: Option<NaiveTime>,
    resume_trade_time: Option<NaiveTime>,
    raw_json: Value,
}

fn parse_nasdaq_halts(body: &str) -> Vec<NasdaqHalt> {
    body.lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() < 10 || parts[0] == "Halt Date" || parts[0].starts_with("File Creation")
            {
                return None;
            }
            let halt_date = parse_date(parts[0])?;
            let symbol = parts[2].trim().to_uppercase();
            if symbol.is_empty() {
                return None;
            }
            Some(NasdaqHalt {
                symbol,
                issue_name: nonempty(parts[3]),
                market: nonempty(parts[4]),
                reason_code: nonempty(parts[5]),
                halt_date,
                halt_time: parse_time(parts[1]),
                resume_date: parse_date(parts[7]),
                resume_quote_time: parse_time(parts[8]),
                resume_trade_time: parse_time(parts[9]),
                raw_json: serde_json::json!({
                    "halt_date": parts[0],
                    "halt_time": parts[1],
                    "symbol": parts[2],
                    "issue_name": parts[3],
                    "market": parts[4],
                    "reason_code": parts[5],
                    "pause_threshold_price": parts[6],
                    "resume_date": parts[7],
                    "resume_quote_time": parts[8],
                    "resume_trade_time": parts[9],
                }),
            })
        })
        .collect()
}

fn nonempty(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn parse_date(value: &str) -> Option<NaiveDate> {
    let value = value.trim();
    NaiveDate::parse_from_str(value, "%m/%d/%Y")
        .or_else(|_| NaiveDate::parse_from_str(value, "%Y-%m-%d"))
        .ok()
}

fn parse_time(value: &str) -> Option<NaiveTime> {
    let value = value.trim();
    NaiveTime::parse_from_str(value, "%H:%M:%S")
        .or_else(|_| NaiveTime::parse_from_str(value, "%H:%M"))
        .ok()
}

fn unix_date(timestamp: Option<i64>) -> Option<NaiveDate> {
    DateTime::<Utc>::from_timestamp(timestamp?, 0).map(|dt| dt.date_naive())
}

fn format_ratio(numerator: f64, denominator: f64) -> String {
    if numerator > 0.0 && denominator > 0.0 {
        format!("{}:{}", numerator as i64, denominator as i64)
    } else {
        "unknown".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nasdaq_trading_halt_rows() {
        let body = "Halt Date|Halt Time|Issue Symbol|Issue Name|Market|Reason Codes|Pause Threshold Price|Resumption Date|Resumption Quote Time|Resumption Trade Time\n07/23/2026|09:44:12|ABCD|Acme Inc|NASDAQ|T1||07/23/2026|10:00:00|10:05:00\nFile Creation Time: 072320261011";
        let rows = parse_nasdaq_halts(body);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].symbol, "ABCD");
        assert_eq!(
            rows[0].halt_date,
            NaiveDate::from_ymd_opt(2026, 7, 23).unwrap()
        );
        assert_eq!(rows[0].reason_code.as_deref(), Some("T1"));
    }

    #[test]
    fn parses_symbol_csv_with_default_fallback() {
        assert_eq!(symbols("spy, qqq"), vec!["SPY", "QQQ"]);
        assert!(symbols(" ").contains(&"SPY".to_string()));
    }
}
