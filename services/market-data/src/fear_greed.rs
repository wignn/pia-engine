use axum::extract::{Query, State};
use axum::Json;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};

use crate::clickhouse::ClickHouseClient;
use crate::config::Config;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct FearGreedRecord {
    pub id: String,
    pub scope: String,
    pub date: DateTime<Utc>,
    pub score: f64,
    pub label: String,
    pub components: serde_json::Value,
    pub source_status: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct FearGreedQuery {
    pub scope: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FearGreedHistoryQuery {
    pub scope: Option<String>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub limit: Option<i64>,
}

pub fn score_label(score: f64) -> &'static str {
    if score < 25.0 {
        "extreme_fear"
    } else if score < 45.0 {
        "fear"
    } else if score < 55.0 {
        "neutral"
    } else if score < 75.0 {
        "greed"
    } else {
        "extreme_greed"
    }
}

pub fn compute_composite_score(
    components: &[(&str, Option<f64>, f64)],
) -> (f64, serde_json::Value) {
    let mut total_weight = 0.0;
    let mut weighted_sum = 0.0;
    let mut status_map = serde_json::Map::new();

    for &(name, score_opt, weight) in components {
        match score_opt {
            Some(score) => {
                total_weight += weight;
                weighted_sum += score * weight;
                status_map.insert(name.to_string(), serde_json::json!("ok"));
            }
            None => {
                status_map.insert(name.to_string(), serde_json::json!("missing"));
            }
        }
    }

    let final_score = if total_weight > 0.0 {
        weighted_sum / total_weight
    } else {
        50.0
    };

    (final_score, serde_json::Value::Object(status_map))
}

pub async fn get_fear_greed_latest(
    Query(query): Query<FearGreedQuery>,
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let scope = query.scope.as_deref().unwrap_or("global");

    let record = sqlx::query_as::<_, FearGreedRecord>(
        "SELECT id, scope, date, score, label, components, source_status, created_at \
         FROM fear_greed_index WHERE scope = $1 ORDER BY date DESC LIMIT 1",
    )
    .bind(scope)
    .fetch_optional(&state.db)
    .await;

    match record {
        Ok(Some(r)) => Json(serde_json::json!(r)),
        Ok(None) => {
            match calculate_and_upsert_fear_greed(&state.db, state.clickhouse.as_deref(), scope)
                .await
            {
                Ok(r) => Json(serde_json::json!(r)),
                Err(err) => {
                    error!(error = %err, "failed to compute fear & greed index");
                    Json(serde_json::json!({ "error": "internal server error" }))
                }
            }
        }
        Err(err) => {
            error!(error = %err, "failed to query fear & greed index");
            Json(serde_json::json!({ "error": "internal server error" }))
        }
    }
}

pub async fn get_fear_greed_history(
    Query(query): Query<FearGreedHistoryQuery>,
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let scope = query.scope.as_deref().unwrap_or("global");
    let limit = query.limit.unwrap_or(30).clamp(1, 365);
    let from_dt = query
        .from
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .map(|dt| dt.and_utc());
    let to_dt = query
        .to
        .and_then(|d| d.and_hms_opt(23, 59, 59))
        .map(|dt| dt.and_utc());

    let records = sqlx::query_as::<_, FearGreedRecord>(
        "SELECT id, scope, date, score, label, components, source_status, created_at \
         FROM fear_greed_index \
         WHERE scope = $1 \
           AND ($2::timestamptz IS NULL OR date >= $2) \
           AND ($3::timestamptz IS NULL OR date <= $3) \
         ORDER BY date DESC \
         LIMIT $4",
    )
    .bind(scope)
    .bind(from_dt)
    .bind(to_dt)
    .bind(limit)
    .fetch_all(&state.db)
    .await;

    match records {
        Ok(rows) => Json(serde_json::json!({
            "data": rows,
            "scope": scope,
            "total": rows.len(),
        })),
        Err(err) => {
            error!(error = %err, "failed to query fear & greed history");
            Json(serde_json::json!({ "error": "internal server error" }))
        }
    }
}

pub async fn get_fear_greed_components(
    Query(query): Query<FearGreedQuery>,
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let scope = query.scope.as_deref().unwrap_or("global");

    let record = sqlx::query_as::<_, FearGreedRecord>(
        "SELECT id, scope, date, score, label, components, source_status, created_at \
         FROM fear_greed_index WHERE scope = $1 ORDER BY date DESC LIMIT 1",
    )
    .bind(scope)
    .fetch_optional(&state.db)
    .await;

    let (score, label, components, source_status, updated_at) = match record {
        Ok(Some(r)) => (r.score, r.label, r.components, r.source_status, r.date),
        _ => {
            let momentum = get_momentum_score(&state.db).await;
            let volatility = get_volatility_score(state.clickhouse.as_deref()).await;
            let safe_haven = get_safe_haven_score(&state.db).await;
            let news_risk = get_news_risk_score(&state.db).await;
            let positioning = get_positioning_score(&state.db).await;

            let defs = vec![
                ("momentum", momentum, 0.25),
                ("volatility", volatility, 0.20),
                ("safe_haven", safe_haven, 0.20),
                ("news_risk", news_risk, 0.20),
                ("positioning", positioning, 0.15),
            ];
            let (sc, st) = compute_composite_score(&defs);
            let lbl = score_label(sc).to_string();
            let mut comp_map = serde_json::Map::new();
            for (name, val_opt, _) in &defs {
                if let Some(val) = val_opt {
                    comp_map.insert(name.to_string(), serde_json::json!(val));
                }
            }
            (sc, lbl, serde_json::Value::Object(comp_map), st, Utc::now())
        }
    };

    let base_weights = [
        (
            "momentum",
            0.25,
            "Short-term market momentum derived from price returns",
        ),
        (
            "volatility",
            0.20,
            "Market volatility and price spike frequency",
        ),
        (
            "safe_haven",
            0.20,
            "Safe haven demand derived from Treasury yield spreads",
        ),
        (
            "news_risk",
            0.20,
            "Geopolitical risk and news sentiment scores",
        ),
        (
            "positioning",
            0.15,
            "Institutional trader positioning from CFTC COT reports",
        ),
    ];

    let items: Vec<serde_json::Value> = base_weights
        .iter()
        .map(|(name, base_weight, desc)| {
            let comp_score = components.get(name).and_then(|v| v.as_f64());
            let status = source_status
                .get(name)
                .and_then(|v| v.as_str())
                .unwrap_or("missing");
            serde_json::json!({
                "name": name,
                "score": comp_score,
                "base_weight": base_weight,
                "status": status,
                "description": desc,
            })
        })
        .collect();

    Json(serde_json::json!({
        "scope": scope,
        "score": score,
        "label": label,
        "updated_at": updated_at,
        "components": items,
    }))
}

pub async fn run_fear_greed_sync(
    cfg: Config,
    pool: PgPool,
    clickhouse: Option<Arc<ClickHouseClient>>,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(cfg.fear_greed_sync_sec));
    loop {
        interval.tick().await;
        info!("updating fear & greed index");
        if let Err(err) =
            calculate_and_upsert_fear_greed(&pool, clickhouse.as_deref(), "global").await
        {
            warn!(error = %err, "fear & greed calculation failed");
        }
    }
}

pub async fn calculate_and_upsert_fear_greed(
    pool: &PgPool,
    clickhouse: Option<&ClickHouseClient>,
    scope: &str,
) -> Result<FearGreedRecord, sqlx::Error> {
    let momentum = get_momentum_score(pool).await;
    let volatility = get_volatility_score(clickhouse).await;
    let safe_haven = get_safe_haven_score(pool).await;
    let news_risk = get_news_risk_score(pool).await;
    let positioning = get_positioning_score(pool).await;

    let components_input = vec![
        ("momentum", momentum, 0.25),
        ("volatility", volatility, 0.20),
        ("safe_haven", safe_haven, 0.20),
        ("news_risk", news_risk, 0.20),
        ("positioning", positioning, 0.15),
    ];

    let (score, source_status) = compute_composite_score(&components_input);
    let label = score_label(score).to_string();

    let mut comp_map = serde_json::Map::new();
    for (name, val_opt, _) in &components_input {
        if let Some(val) = val_opt {
            comp_map.insert(name.to_string(), serde_json::json!(val));
        }
    }
    let components = serde_json::Value::Object(comp_map);

    let date = Utc::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc();
    let id = format!("{scope}_{}", date.format("%Y-%m-%d"));

    let record = sqlx::query_as::<_, FearGreedRecord>(
        r#"
        INSERT INTO fear_greed_index (id, scope, date, score, label, components, source_status)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (scope, date) DO UPDATE SET
            score = EXCLUDED.score,
            label = EXCLUDED.label,
            components = EXCLUDED.components,
            source_status = EXCLUDED.source_status
        RETURNING id, scope, date, score, label, components, source_status, created_at
        "#,
    )
    .bind(&id)
    .bind(scope)
    .bind(date)
    .bind(score)
    .bind(&label)
    .bind(&components)
    .bind(&source_status)
    .fetch_one(pool)
    .await?;

    Ok(record)
}

async fn get_momentum_score(pool: &PgPool) -> Option<f64> {
    let row: Option<(Option<f64>,)> = sqlx::query_as(
        r#"
        WITH ranked AS (
            SELECT
                symbol,
                close,
                first_value(close) OVER (PARTITION BY symbol ORDER BY time ASC) AS first_close,
                row_number() OVER (PARTITION BY symbol ORDER BY time DESC) AS latest_rank
            FROM market.ohlcv_candles
            WHERE resolution = '1m'
              AND time >= NOW() - INTERVAL '1 day'
              AND close > 0
        )
        SELECT AVG((close - first_close) / NULLIF(first_close, 0))
        FROM ranked
        WHERE latest_rank = 1
        "#,
    )
    .fetch_optional(pool)
    .await
    .ok()?;

    let avg_return = row.and_then(|(value,)| value)?;
    Some((50.0 + avg_return * 500.0).clamp(0.0, 100.0))
}

async fn get_volatility_score(clickhouse: Option<&ClickHouseClient>) -> Option<f64> {
    if let Some(ch) = clickhouse {
        if let Ok(spikes) = ch.spike_candidates(60, 0.35, 100).await {
            let count = spikes.len() as f64;
            let score = (100.0 - count * 5.0).clamp(0.0, 100.0);
            return Some(score);
        }
    }
    None
}

async fn get_safe_haven_score(pool: &PgPool) -> Option<f64> {
    let row: Option<(f64,)> = sqlx::query_as(
        "SELECT value FROM macro_rates WHERE country = 'US' AND tenor = '10Y' ORDER BY date DESC LIMIT 1"
    )
    .fetch_optional(pool)
    .await
    .ok()?;

    if let Some((val,)) = row {
        let score = (100.0 - val * 10.0).clamp(0.0, 100.0);
        return Some(score);
    }
    None
}

async fn get_news_risk_score(pool: &PgPool) -> Option<f64> {
    let row: Option<(Option<f64>, Option<f64>)> = sqlx::query_as(
        "SELECT AVG(sentiment_score), AVG(severity_score) FROM news.geosignals WHERE timestamp >= NOW() - INTERVAL '24 hours'"
    )
    .fetch_optional(pool)
    .await
    .ok()?;

    if let Some((sentiment_opt, severity_opt)) = row {
        if let (Some(sent), Some(sev)) = (sentiment_opt, severity_opt) {
            let score = ((sent + 1.0) / 2.0 * 50.0 + (1.0 - sev) * 50.0).clamp(0.0, 100.0);
            return Some(score);
        }
    }
    None
}

async fn get_positioning_score(pool: &PgPool) -> Option<f64> {
    let row: Option<(Option<i64>, Option<i64>)> = sqlx::query_as(
        "SELECT SUM(noncommercial_long), SUM(noncommercial_short) FROM cot_reports WHERE report_date >= NOW() - INTERVAL '30 days'"
    )
    .fetch_optional(pool)
    .await
    .ok()?;

    if let Some((long_opt, short_opt)) = row {
        if let (Some(l), Some(s)) = (long_opt, short_opt) {
            let total = l + s;
            if total > 0 {
                let score = (l as f64 / total as f64 * 100.0).clamp(0.0, 100.0);
                return Some(score);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_label_from_score() {
        assert_eq!(score_label(15.0), "extreme_fear");
        assert_eq!(score_label(35.0), "fear");
        assert_eq!(score_label(50.0), "neutral");
        assert_eq!(score_label(65.0), "greed");
        assert_eq!(score_label(85.0), "extreme_greed");
    }

    #[test]
    fn rebalances_weights_when_component_missing() {
        let components = vec![
            ("momentum", Some(60.0), 0.25),
            ("volatility", Some(40.0), 0.20),
            ("safe_haven", None, 0.20),
            ("news_risk", Some(50.0), 0.20),
            ("positioning", Some(50.0), 0.15),
        ];
        let (score, status) = compute_composite_score(&components);
        assert!((score - 50.625).abs() < 0.1);
        assert_eq!(status["safe_haven"], "missing");
    }
}
