use sqlx::SqlitePool;

pub async fn exists(pool: &SqlitePool, item_id: &str) -> Result<bool, sqlx::Error> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sent_items WHERE item_id = ?")
        .bind(item_id)
        .fetch_one(pool)
        .await?;

    Ok(count.0 > 0)
}

pub async fn exists_with_type(
    pool: &SqlitePool,
    item_id: &str,
    item_type: &str,
) -> Result<bool, sqlx::Error> {
    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM sent_items WHERE item_id = ? AND item_type = ?")
            .bind(item_id)
            .bind(item_type)
            .fetch_one(pool)
            .await?;

    Ok(count.0 > 0)
}

pub async fn insert(
    pool: &SqlitePool,
    item_id: &str,
    item_type: &str,
    source: &str,
) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().timestamp();
    sqlx::query(
        "INSERT INTO sent_items (item_id, item_type, source, sent_at) VALUES (?, ?, ?, ?) ON CONFLICT(item_id) DO NOTHING",
    )
    .bind(item_id)
    .bind(item_type)
    .bind(source)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn cleanup(pool: &SqlitePool, item_type: &str, days: i64) -> Result<u64, sqlx::Error> {
    let cutoff = chrono::Utc::now().timestamp() - (days * 86400);
    let result = sqlx::query("DELETE FROM sent_items WHERE item_type = ? AND sent_at < ?")
        .bind(item_type)
        .bind(cutoff)
        .execute(pool)
        .await?;

    Ok(result.rows_affected())
}
