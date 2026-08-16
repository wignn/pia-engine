use chrono::{DateTime, Utc};
use sqlx::PgPool;
use tracing::{error, warn};

use crate::prices::CachedPrice;

pub async fn record_tick_batch(
    pool: &PgPool,
    batch: &[(CachedPrice, DateTime<Utc>)],
    err: &anyhow::Error,
) {
    let payload = match serde_json::to_value(
        batch
            .iter()
            .map(|(price, received_at)| serde_json::json!({ "tick": price, "received_at": received_at }))
            .collect::<Vec<_>>(),
    ) {
        Ok(payload) => payload,
        Err(err) => {
            error!(error = %err, batch_size = batch.len(), "failed to serialize dead-letter tick batch");
            return;
        }
    };

    let result = sqlx::query(
        "INSERT INTO platform.deadletter_batches (target, payload, error, batch_size) VALUES ($1, $2, $3, $4)",
    )
    .bind("clickhouse.price_ticks")
    .bind(payload)
    .bind(err.to_string())
    .bind(batch.len() as i32)
    .execute(pool)
    .await;

    match result {
        Ok(_) => warn!(
            batch_size = batch.len(),
            "tick batch recorded in dead letter; replay with scripts/backfill-ohlcv-from-ticks.sql after recovery"
        ),
        Err(db_err) => error!(
            error = %db_err,
            original_error = %err,
            batch_size = batch.len(),
            "dead-letter insert failed; tick batch is LOST"
        ),
    }
}
