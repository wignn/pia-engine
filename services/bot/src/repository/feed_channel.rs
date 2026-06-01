use sqlx::SqlitePool;

pub async fn upsert_by_guild(
    pool: &SqlitePool,
    table: &str,
    guild_id: u64,
    channel_id: u64,
) -> Result<(), sqlx::Error> {
    let query = format!(
        "INSERT INTO {table} (guild_id, channel_id, is_active)
         VALUES (?, ?, 1)
         ON CONFLICT(guild_id) DO UPDATE SET channel_id = excluded.channel_id, is_active = 1"
    );

    sqlx::query(&query)
        .bind(guild_id as i64)
        .bind(channel_id as i64)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn set_active_by_guild(
    pool: &SqlitePool,
    table: &str,
    guild_id: u64,
    is_active: bool,
) -> Result<(), sqlx::Error> {
    let query = format!("UPDATE {table} SET is_active = ? WHERE guild_id = ?");

    sqlx::query(&query)
        .bind(is_active)
        .bind(guild_id as i64)
        .execute(pool)
        .await?;

    Ok(())
}
