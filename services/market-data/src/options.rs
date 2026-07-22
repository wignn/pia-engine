use axum::extract::{Query, State};
use axum::Json;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{error, info, warn};

use atlsd_eventbus::{subjects, EventBusMode};
use futures_util::StreamExt;

use crate::state::AppState;

fn norm_cdf(x: f64) -> f64 {
    0.5 * (1.0 + erf(x / std::f64::consts::SQRT_2))
}

fn norm_pdf(x: f64) -> f64 {
    (-0.5 * x * x).exp() / (2.0 * std::f64::consts::PI).sqrt()
}

fn erf(x: f64) -> f64 {
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;

    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x_abs = x.abs();
    let t = 1.0 / (1.0 + p * x_abs);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x_abs * x_abs).exp();

    sign * y
}

fn calculate_greeks(
    option_type: &str,
    strike: f64,
    underlying_price: f64,
    time_years: f64,
    iv: f64,
    risk_free_rate: f64,
) -> (f64, f64, f64, f64) {
    if time_years <= 0.0 || iv <= 0.0 || underlying_price <= 0.0 || strike <= 0.0 {
        return (
            contract_delta_at_expiry(option_type, strike, underlying_price),
            0.0,
            0.0,
            0.0,
        );
    }

    let sqrt_t = time_years.sqrt();
    let d1 = ((underlying_price / strike).ln() + (risk_free_rate + 0.5 * iv * iv) * time_years)
        / (iv * sqrt_t);
    let d2 = d1 - iv * sqrt_t;
    let n_d1 = norm_cdf(d1);
    let n_prime_d1 = norm_pdf(d1);

    let (delta, theta) = match option_type.to_lowercase().as_str() {
        "call" => (
            n_d1,
            -(underlying_price * n_prime_d1 * iv) / (2.0 * sqrt_t)
                - risk_free_rate * strike * (-risk_free_rate * time_years).exp() * norm_cdf(d2),
        ),
        "put" => (
            n_d1 - 1.0,
            -(underlying_price * n_prime_d1 * iv) / (2.0 * sqrt_t)
                + risk_free_rate * strike * (-risk_free_rate * time_years).exp() * norm_cdf(-d2),
        ),
        _ => (0.0, 0.0),
    };

    let gamma = n_prime_d1 / (underlying_price * iv * sqrt_t);
    let vega = underlying_price * n_prime_d1 * sqrt_t;

    (delta, gamma, theta, vega)
}

fn contract_delta_at_expiry(option_type: &str, strike: f64, underlying_price: f64) -> f64 {
    match option_type.to_lowercase().as_str() {
        "call" if underlying_price > strike => 1.0,
        "put" if underlying_price < strike => -1.0,
        _ => 0.0,
    }
}

fn calculate_gex(gamma: f64, underlying_price: f64, open_interest: i64, is_call: bool) -> f64 {
    let sign = if is_call { 1.0 } else { -1.0 };
    sign * gamma * underlying_price * underlying_price * 100.0 * open_interest as f64
}

fn option_time_years(expiration_date: NaiveDate) -> f64 {
    let days = expiration_date
        .signed_duration_since(Utc::now().date_naive())
        .num_days() as f64;
    (days / 365.25).max(0.001)
}

fn calculate_max_pain(contracts: &[OptionContractPayload]) -> f64 {
    let mut strikes: Vec<f64> = contracts.iter().map(|contract| contract.strike).collect();
    strikes.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    strikes.dedup();

    strikes
        .into_iter()
        .min_by(|left, right| {
            payout_at(*left, contracts)
                .partial_cmp(&payout_at(*right, contracts))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or(0.0)
}

fn payout_at(strike: f64, contracts: &[OptionContractPayload]) -> f64 {
    contracts
        .iter()
        .map(|contract| {
            let oi = contract.open_interest as f64;
            if contract.option_type.eq_ignore_ascii_case("call") && strike > contract.strike {
                (strike - contract.strike) * oi
            } else if contract.option_type.eq_ignore_ascii_case("put") && strike < contract.strike {
                (contract.strike - strike) * oi
            } else {
                0.0
            }
        })
        .sum()
}

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

    let underlying_price = payload.underlying_price.unwrap_or(0.0);
    let mut total_open_interest = 0_i64;
    let mut total_volume = 0_i64;
    let mut total_gex = 0.0;
    let mut call_volume = 0_i64;
    let mut put_volume = 0_i64;
    let mut iv_atm: Option<(f64, f64)> = None;

    for contract in &payload.contracts {
        let expiration_date = NaiveDate::parse_from_str(&contract.expiration_date, "%Y-%m-%d")
            .unwrap_or_else(|_| Utc::now().date_naive());
        let (delta, gamma, theta, vega) = calculate_greeks(
            &contract.option_type,
            contract.strike,
            underlying_price,
            option_time_years(expiration_date),
            contract.implied_volatility,
            0.045,
        );
        let gex = calculate_gex(
            gamma,
            underlying_price,
            contract.open_interest,
            contract.option_type.eq_ignore_ascii_case("call"),
        );
        total_open_interest += contract.open_interest;
        total_volume += contract.volume;
        total_gex += gex;
        if contract.option_type.eq_ignore_ascii_case("call") {
            call_volume += contract.volume;
        } else if contract.option_type.eq_ignore_ascii_case("put") {
            put_volume += contract.volume;
        }
        let atm_distance = (contract.strike - underlying_price).abs();
        if iv_atm
            .map(|(distance, _)| atm_distance < distance)
            .unwrap_or(true)
        {
            iv_atm = Some((atm_distance, contract.implied_volatility));
        }

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
        .bind(delta)
        .bind(gamma)
        .bind(theta)
        .bind(vega)
        .bind(gex)
        .bind(contract.open_interest)
        .bind(contract.volume)
        .bind(updated_at)
        .execute(pool)
        .await?;
    }

    if !payload.contracts.is_empty() {
        let summary = OptionsSummaryPayload {
            id: Some(payload.symbol.clone()),
            symbol: payload.symbol.clone(),
            underlying_price,
            put_call_ratio: if call_volume == 0 {
                0.0
            } else {
                put_volume as f64 / call_volume as f64
            },
            max_pain_strike: calculate_max_pain(&payload.contracts),
            total_open_interest,
            total_volume,
            total_gex,
            iv_atm: iv_atm.map(|(_, iv)| iv),
            updated_at: payload.updated_at.clone(),
        };
        upsert_options_summary(pool, &summary).await?;
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
    subscribers.push(
        client
            .subscribe(subjects::MARKET_OPTIONS_SUMMARY_V1.to_string())
            .await?,
    );
    subscribers.push(
        client
            .subscribe(subjects::MARKET_OPTIONS_CHAIN_V1.to_string())
            .await?,
    );
    subscribers.push(
        client
            .subscribe(subjects::MD_RAW_OPTIONS_SUMMARY_V1.to_string())
            .await?,
    );
    subscribers.push(
        client
            .subscribe(subjects::MD_RAW_OPTIONS_CHAIN_V1.to_string())
            .await?,
    );
    info!("connected to NATS market.options.* subjects");

    while let Some(message) = subscribers.next().await {
        if let Ok(payload) = std::str::from_utf8(&message.payload) {
            handle_options_payload(payload, &state.db).await;
        }
    }

    Ok(())
}

pub async fn handle_options_payload(payload: &str, pool: &PgPool) {
    match serde_json::from_str::<OptionsSummaryPayload>(payload) {
        Ok(summary)
            if summary.put_call_ratio >= 0.0
                && !summary.symbol.is_empty()
                && summary.max_pain_strike >= 0.0 =>
        {
            if let Err(err) = upsert_options_summary(pool, &summary).await {
                error!(error = %err, symbol = %summary.symbol, "failed to upsert options summary");
            } else {
                info!(symbol = %summary.symbol, "upserted options summary");
            }
            return;
        }
        Ok(summary) => {
            warn!(symbol = %summary.symbol, "ignored invalid options summary payload");
        }
        Err(_) => {}
    }

    match serde_json::from_str::<OptionsChainPayload>(payload) {
        Ok(chain) if !chain.symbol.is_empty() => {
            let contract_count = chain.contracts.len();
            if let Err(err) = upsert_options_chain(pool, &chain).await {
                error!(error = %err, symbol = %chain.symbol, contract_count, "failed to upsert options chain");
            } else {
                info!(symbol = %chain.symbol, contract_count, "upserted options chain");
            }
        }
        Ok(_) => warn!("ignored options chain payload with empty symbol"),
        Err(err) => warn!(error = %err, "payload is not an options chain"),
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
