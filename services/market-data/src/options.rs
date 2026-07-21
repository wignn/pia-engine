use axum::extract::{Query, State};
use axum::Json;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{error, info, warn};

use atlsd_eventbus::{subjects, EventBusMode};
use futures_util::StreamExt;

use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OptionsSnapshotRow {
    pub id: String,
    pub symbol: String,
    pub underlying_price: f64,
    pub put_call_ratio: f64,
    pub max_pain_strike: f64,
    pub total_open_interest: i64,
    pub total_volume: i64,
    pub total_gex: f64,
    pub iv_atm: Option<f64>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OptionsContractRow {
    pub contract_symbol: String,
    pub symbol: String,
    pub option_type: String,
    pub strike: f64,
    pub expiration_date: NaiveDate,
    pub mark_price: f64,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub implied_volatility: f64,
    pub delta: f64,
    pub gamma: f64,
    pub theta: f64,
    pub vega: f64,
    pub gex: f64,
    pub open_interest: i64,
    pub volume: i64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OptionsGexRow {
    pub strike: f64,
    pub call_gex: f64,
    pub put_gex: f64,
    pub total_gex: f64,
}

#[derive(Debug, Deserialize)]
pub struct OptionsSummaryQuery {
    pub symbol: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OptionsChainQuery {
    pub symbol: Option<String>,
    pub expiration_date: Option<NaiveDate>,
    pub option_type: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct OptionsGexQuery {
    pub symbol: Option<String>,
    pub expiration_date: Option<NaiveDate>,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionsSummaryPayload {
    pub id: Option<String>,
    pub symbol: String,
    pub underlying_price: f64,
    pub put_call_ratio: f64,
    pub max_pain_strike: f64,
    pub total_open_interest: i64,
    pub total_volume: i64,
    pub total_gex: f64,
    pub iv_atm: Option<f64>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionContractPayload {
    pub contract_symbol: String,
    pub symbol: String,
    pub option_type: String,
    pub strike: f64,
    pub expiration_date: String,
    pub mark_price: f64,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub implied_volatility: f64,
    pub delta: f64,
    pub gamma: f64,
    pub theta: f64,
    pub vega: f64,
    pub gex: f64,
    pub open_interest: i64,
    pub volume: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionsChainPayload {
    pub symbol: String,
    pub underlying_price: Option<f64>,
    pub contracts: Vec<OptionContractPayload>,
    pub updated_at: Option<String>,
}

// Database Queries

pub async fn query_options_summary(
    pool: &PgPool,
    symbol: Option<&str>,
) -> Result<Vec<OptionsSnapshotRow>, sqlx::Error> {
    sqlx::query_as::<_, OptionsSnapshotRow>(
        r#"
        SELECT id, symbol, underlying_price, put_call_ratio, max_pain_strike,
               total_open_interest, total_volume, total_gex, iv_atm, updated_at
        FROM options_snapshots
        WHERE ($1::text IS NULL OR UPPER(symbol) = UPPER($1))
        ORDER BY symbol ASC
        "#,
    )
    .bind(symbol)
    .fetch_all(pool)
    .await
}

pub async fn query_options_chain(
    pool: &PgPool,
    symbol: Option<&str>,
    expiration_date: Option<NaiveDate>,
    option_type: Option<&str>,
    limit: i64,
) -> Result<Vec<OptionsContractRow>, sqlx::Error> {
    sqlx::query_as::<_, OptionsContractRow>(
        r#"
        SELECT contract_symbol, symbol, option_type, strike, expiration_date,
               mark_price, bid, ask, implied_volatility, delta, gamma, theta, vega,
               gex, open_interest, volume, updated_at
        FROM options_contracts
        WHERE ($1::text IS NULL OR UPPER(symbol) = UPPER($1))
          AND ($2::date IS NULL OR expiration_date = $2)
          AND ($3::text IS NULL OR LOWER(option_type) = LOWER($3))
        ORDER BY expiration_date ASC, strike ASC
        LIMIT $4
        "#,
    )
    .bind(symbol)
    .bind(expiration_date)
    .bind(option_type)
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn query_options_gex(
    pool: &PgPool,
    symbol: Option<&str>,
    expiration_date: Option<NaiveDate>,
    limit: i64,
) -> Result<Vec<OptionsGexRow>, sqlx::Error> {
    sqlx::query_as::<_, OptionsGexRow>(
        r#"
        SELECT
            strike,
            COALESCE(SUM(CASE WHEN LOWER(option_type) = 'call' THEN gex ELSE 0 END), 0.0) AS call_gex,
            COALESCE(SUM(CASE WHEN LOWER(option_type) = 'put' THEN gex ELSE 0 END), 0.0) AS put_gex,
            COALESCE(SUM(gex), 0.0) AS total_gex
        FROM options_contracts
        WHERE ($1::text IS NULL OR UPPER(symbol) = UPPER($1))
          AND ($2::date IS NULL OR expiration_date = $2)
        GROUP BY strike
        ORDER BY strike ASC
        LIMIT $3
        "#,
    )
    .bind(symbol)
    .bind(expiration_date)
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn upsert_options_summary(
    pool: &PgPool,
    payload: &OptionsSummaryPayload,
) -> Result<(), sqlx::Error> {
    let id = payload.id.clone().unwrap_or_else(|| payload.symbol.clone());
    let updated_at = payload
        .updated_at
        .as_deref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(Utc::now);

    sqlx::query(
        r#"
        INSERT INTO options_snapshots (
            id, symbol, underlying_price, put_call_ratio, max_pain_strike,
            total_open_interest, total_volume, total_gex, iv_atm, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        ON CONFLICT (id) DO UPDATE SET
            symbol = EXCLUDED.symbol,
            underlying_price = EXCLUDED.underlying_price,
            put_call_ratio = EXCLUDED.put_call_ratio,
            max_pain_strike = EXCLUDED.max_pain_strike,
            total_open_interest = EXCLUDED.total_open_interest,
            total_volume = EXCLUDED.total_volume,
            total_gex = EXCLUDED.total_gex,
            iv_atm = EXCLUDED.iv_atm,
            updated_at = EXCLUDED.updated_at
        "#,
    )
    .bind(id)
    .bind(&payload.symbol)
    .bind(payload.underlying_price)
    .bind(payload.put_call_ratio)
    .bind(payload.max_pain_strike)
    .bind(payload.total_open_interest)
    .bind(payload.total_volume)
    .bind(payload.total_gex)
    .bind(payload.iv_atm)
    .bind(updated_at)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn upsert_options_chain(
    pool: &PgPool,
    payload: &OptionsChainPayload,
) -> Result<(), sqlx::Error> {
    let updated_at = payload
        .updated_at
        .as_deref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(Utc::now);

    for contract in &payload.contracts {
        let expiration_date = NaiveDate::parse_from_str(&contract.expiration_date, "%Y-%m-%d")
            .unwrap_or_else(|_| Utc::now().date_naive());

        sqlx::query(
            r#"
            INSERT INTO options_contracts (
                contract_symbol, symbol, option_type, strike, expiration_date,
                mark_price, bid, ask, implied_volatility, delta, gamma, theta, vega,
                gex, open_interest, volume, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
            ON CONFLICT (contract_symbol) DO UPDATE SET
                symbol = EXCLUDED.symbol,
                option_type = EXCLUDED.option_type,
                strike = EXCLUDED.strike,
                expiration_date = EXCLUDED.expiration_date,
                mark_price = EXCLUDED.mark_price,
                bid = EXCLUDED.bid,
                ask = EXCLUDED.ask,
                implied_volatility = EXCLUDED.implied_volatility,
                delta = EXCLUDED.delta,
                gamma = EXCLUDED.gamma,
                theta = EXCLUDED.theta,
                vega = EXCLUDED.vega,
                gex = EXCLUDED.gex,
                open_interest = EXCLUDED.open_interest,
                volume = EXCLUDED.volume,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(&contract.contract_symbol)
        .bind(&contract.symbol)
        .bind(&contract.option_type)
        .bind(contract.strike)
        .bind(expiration_date)
        .bind(contract.mark_price)
        .bind(contract.bid)
        .bind(contract.ask)
        .bind(contract.implied_volatility)
        .bind(contract.delta)
        .bind(contract.gamma)
        .bind(contract.theta)
        .bind(contract.vega)
        .bind(contract.gex)
        .bind(contract.open_interest)
        .bind(contract.volume)
        .bind(updated_at)
        .execute(pool)
        .await?;
    }

    Ok(())
}

// Axum Handlers

pub async fn get_options_summary(
    State(state): State<AppState>,
    Query(params): Query<OptionsSummaryQuery>,
) -> Json<serde_json::Value> {
    match query_options_summary(&state.db, params.symbol.as_deref()).await {
        Ok(snapshots) => Json(serde_json::json!({ "data": snapshots })),
        Err(err) => {
            error!(error = %err, "failed to query options summary");
            Json(serde_json::json!({ "error": "internal server error" }))
        }
    }
}

pub async fn get_options_chain(
    State(state): State<AppState>,
    Query(params): Query<OptionsChainQuery>,
) -> Json<serde_json::Value> {
    let limit = params.limit.unwrap_or(100).clamp(1, 1000);
    match query_options_chain(
        &state.db,
        params.symbol.as_deref(),
        params.expiration_date,
        params.option_type.as_deref(),
        limit,
    )
    .await
    {
        Ok(contracts) => Json(serde_json::json!({ "data": contracts })),
        Err(err) => {
            error!(error = %err, "failed to query options chain");
            Json(serde_json::json!({ "error": "internal server error" }))
        }
    }
}

pub async fn get_options_gex(
    State(state): State<AppState>,
    Query(params): Query<OptionsGexQuery>,
) -> Json<serde_json::Value> {
    let limit = params.limit.unwrap_or(100).clamp(1, 1000);
    match query_options_gex(
        &state.db,
        params.symbol.as_deref(),
        params.expiration_date,
        limit,
    )
    .await
    {
        Ok(gex) => Json(serde_json::json!({ "data": gex })),
        Err(err) => {
            error!(error = %err, "failed to query options gex");
            Json(serde_json::json!({ "error": "internal server error" }))
        }
    }
}

// Background Subscriber / Sync Function

pub async fn run_options_subscriber(state: AppState) {
    info!(sync_sec = state.config.options_sync_sec, mode = %state.config.eventbus_mode, "starting options eventbus subscriber");
    match EventBusMode::from_env_value(&state.config.eventbus_mode) {
        EventBusMode::Nats => run_nats(state).await,
        EventBusMode::Dual => {
            let redis_state = state.clone();
            tokio::spawn(async move {
                run_redis(redis_state).await;
            });
            run_nats(state).await;
        }
        EventBusMode::Redis => run_redis(state).await,
        EventBusMode::Noop => warn!("options eventbus subscriber disabled; EVENTBUS_MODE=noop"),
    }
}

async fn run_redis(state: AppState) {
    if !state.config.has_redis() {
        warn!("options Redis subscriber disabled; REDIS_URL is empty");
        return;
    }

    loop {
        if let Err(err) = subscribe_redis_loop(&state).await {
            error!(error = %err, "options Redis subscriber error, reconnecting in 5s");
        }
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}

async fn subscribe_redis_loop(state: &AppState) -> anyhow::Result<()> {
    let client = redis::Client::open(state.config.redis_url.clone())?;
    let mut pubsub = client.get_async_pubsub().await?;
    pubsub.psubscribe("market.options.*").await?;
    info!("connected to market.options.* redis pubsub");

    while let Some(message) = pubsub.on_message().next().await {
        let payload: String = message.get_payload()?;
        handle_options_payload(&payload, &state.db).await;
    }

    Ok(())
}

async fn run_nats(state: AppState) {
    loop {
        if let Err(err) = subscribe_nats_loop(&state).await {
            error!(error = %err, "options NATS subscriber error, reconnecting in 5s");
        }
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}

async fn subscribe_nats_loop(state: &AppState) -> anyhow::Result<()> {
    let client = async_nats::connect(&state.config.nats_url).await?;
    atlsd_eventbus::nats::init_jetstream_streams(&client).await?;
    let mut subscribers = futures_util::stream::SelectAll::new();
    subscribers.push(client.subscribe(subjects::MARKET_OPTIONS_SUMMARY_V1.to_string()).await?);
    subscribers.push(client.subscribe(subjects::MARKET_OPTIONS_CHAIN_V1.to_string()).await?);
    info!("connected to NATS market.options.* subjects");

    while let Some(message) = subscribers.next().await {
        if let Ok(payload) = std::str::from_utf8(&message.payload) {
            handle_options_payload(payload, &state.db).await;
        }
    }

    Ok(())
}

pub async fn handle_options_payload(payload: &str, pool: &PgPool) {
    if let Ok(summary) = serde_json::from_str::<OptionsSummaryPayload>(payload) {
        if summary.put_call_ratio >= 0.0 && !summary.symbol.is_empty() && summary.max_pain_strike >= 0.0 {
            if let Err(err) = upsert_options_summary(pool, &summary).await {
                error!(error = %err, symbol = %summary.symbol, "failed to upsert options summary");
            }
            return;
        }
    }

    if let Ok(chain) = serde_json::from_str::<OptionsChainPayload>(payload) {
        if !chain.symbol.is_empty() {
            if let Err(err) = upsert_options_chain(pool, &chain).await {
                error!(error = %err, symbol = %chain.symbol, "failed to upsert options chain");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_query_param_limit_bounds() {
        let q1: OptionsChainQuery = serde_json::from_value(json!({ "limit": 500 })).unwrap();
        assert_eq!(q1.limit.unwrap_or(100).clamp(1, 1000), 500);

        let q2: OptionsChainQuery = serde_json::from_value(json!({ "limit": 5000 })).unwrap();
        assert_eq!(q2.limit.unwrap_or(100).clamp(1, 1000), 1000);

        let q3: OptionsChainQuery = serde_json::from_value(json!({})).unwrap();
        assert_eq!(q3.limit.unwrap_or(100).clamp(1, 1000), 100);

        let q4: OptionsChainQuery = serde_json::from_value(json!({ "limit": -10 })).unwrap();
        assert_eq!(q4.limit.unwrap_or(100).clamp(1, 1000), 1);
    }

    #[test]
    fn test_parse_options_summary_payload() {
        let raw = json!({
            "id": "BTC",
            "symbol": "BTC",
            "underlying_price": 65000.0,
            "put_call_ratio": 0.85,
            "max_pain_strike": 64000.0,
            "total_open_interest": 12000,
            "total_volume": 4500,
            "total_gex": 1500000.0,
            "iv_atm": 0.55,
            "updated_at": "2026-07-22T00:00:00Z"
        });

        let summary: OptionsSummaryPayload = serde_json::from_value(raw).unwrap();
        assert_eq!(summary.symbol, "BTC");
        assert_eq!(summary.underlying_price, 65000.0);
        assert_eq!(summary.max_pain_strike, 64000.0);
    }

    #[test]
    fn test_parse_options_chain_payload() {
        let raw = json!({
            "symbol": "BTC",
            "underlying_price": 65000.0,
            "contracts": [
                {
                    "contract_symbol": "BTC-26JUL26-65000-C",
                    "symbol": "BTC",
                    "option_type": "call",
                    "strike": 65000.0,
                    "expiration_date": "2026-07-26",
                    "mark_price": 1200.0,
                    "bid": 1190.0,
                    "ask": 1210.0,
                    "implied_volatility": 0.55,
                    "delta": 0.51,
                    "gamma": 0.0001,
                    "theta": -15.0,
                    "vega": 25.0,
                    "gex": 50000.0,
                    "open_interest": 150,
                    "volume": 45
                }
            ],
            "updated_at": "2026-07-22T00:00:00Z"
        });

        let chain: OptionsChainPayload = serde_json::from_value(raw).unwrap();
        assert_eq!(chain.symbol, "BTC");
        assert_eq!(chain.contracts.len(), 1);
        assert_eq!(chain.contracts[0].contract_symbol, "BTC-26JUL26-65000-C");
        assert_eq!(chain.contracts[0].strike, 65000.0);
    }
}
