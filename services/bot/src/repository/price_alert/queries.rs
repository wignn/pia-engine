use super::{PriceAlert, PriceAlertRow};
use sqlx::SqlitePool;

const SELECT_PRICE_ALERT: &str = "SELECT id, user_id, guild_id, symbol, target_price, direction, is_triggered, created_at, triggered_at";

pub async fn create(
    pool: &SqlitePool,
    user_id: u64,
    guild_id: u64,
    symbol: &str,
    target_price: f64,
    direction: &str,
) -> Result<PriceAlert, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO price_alerts (user_id, guild_id, symbol, target_price, direction) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(user_id as i64)
    .bind(guild_id as i64)
    .bind(symbol)
    .bind(target_price)
    .bind(direction)
    .execute(pool)
    .await?;

    get_by_id(pool, result.last_insert_rowid()).await
}

pub async fn get_user_active(
    pool: &SqlitePool,
    user_id: u64,
) -> Result<Vec<PriceAlert>, sqlx::Error> {
    let query = format!(
        "{SELECT_PRICE_ALERT} FROM price_alerts WHERE user_id = ? AND is_triggered = 0 ORDER BY created_at DESC"
    );
    fetch_alerts(pool, &query, user_id as i64).await
}

pub async fn get_active_by_symbol(
    pool: &SqlitePool,
    symbol: &str,
) -> Result<Vec<PriceAlert>, sqlx::Error> {
    let query =
        format!("{SELECT_PRICE_ALERT} FROM price_alerts WHERE symbol = ? AND is_triggered = 0");
    let rows = sqlx::query_as::<_, PriceAlertRow>(&query)
        .bind(symbol)
        .fetch_all(pool)
        .await?;

    Ok(rows.into_iter().map(PriceAlert::from).collect())
}

pub async fn get_all_active_symbols(pool: &SqlitePool) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (String,)>(
        "SELECT DISTINCT symbol FROM price_alerts WHERE is_triggered = 0",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| r.0).collect())
}

pub async fn trigger(pool: &SqlitePool, alert_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE price_alerts SET is_triggered = 1, triggered_at = datetime('now') WHERE id = ?",
    )
    .bind(alert_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete(pool: &SqlitePool, alert_id: i64, user_id: u64) -> Result<bool, sqlx::Error> {
    let result =
        sqlx::query("DELETE FROM price_alerts WHERE id = ? AND user_id = ? AND is_triggered = 0")
            .bind(alert_id)
            .bind(user_id as i64)
            .execute(pool)
            .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn count_user_active(pool: &SqlitePool, user_id: u64) -> Result<i64, sqlx::Error> {
    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM price_alerts WHERE user_id = ? AND is_triggered = 0")
            .bind(user_id as i64)
            .fetch_one(pool)
            .await?;

    Ok(count.0)
}

async fn get_by_id(pool: &SqlitePool, id: i64) -> Result<PriceAlert, sqlx::Error> {
    let query = format!("{SELECT_PRICE_ALERT} FROM price_alerts WHERE id = ?");
    let row = sqlx::query_as::<_, PriceAlertRow>(&query)
        .bind(id)
        .fetch_one(pool)
        .await?;

    Ok(PriceAlert::from(row))
}

async fn fetch_alerts(
    pool: &SqlitePool,
    query: &str,
    user_id: i64,
) -> Result<Vec<PriceAlert>, sqlx::Error> {
    let rows = sqlx::query_as::<_, PriceAlertRow>(query)
        .bind(user_id)
        .fetch_all(pool)
        .await?;

    Ok(rows.into_iter().map(PriceAlert::from).collect())
}
