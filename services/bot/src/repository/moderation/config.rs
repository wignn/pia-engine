use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct ModConfig {
    pub guild_id: i64,
    pub auto_role_id: Option<i64>,
    pub log_channel_id: Option<i64>,
}

pub async fn get(pool: &SqlitePool, guild_id: u64) -> Result<Option<ModConfig>, sqlx::Error> {
    let row = sqlx::query_as::<_, (i64, Option<i64>, Option<i64>)>(
        "SELECT guild_id, auto_role_id, log_channel_id FROM mod_config WHERE guild_id = ?",
    )
    .bind(guild_id as i64)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(ModConfig::from))
}

pub async fn set_auto_role(
    pool: &SqlitePool,
    guild_id: u64,
    role_id: u64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO mod_config (guild_id, auto_role_id) VALUES (?, ?)
         ON CONFLICT(guild_id) DO UPDATE SET auto_role_id = excluded.auto_role_id",
    )
    .bind(guild_id as i64)
    .bind(role_id as i64)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn disable_auto_role(pool: &SqlitePool, guild_id: u64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE mod_config SET auto_role_id = NULL WHERE guild_id = ?")
        .bind(guild_id as i64)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn set_log_channel(
    pool: &SqlitePool,
    guild_id: u64,
    channel_id: u64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO mod_config (guild_id, log_channel_id) VALUES (?, ?)
         ON CONFLICT(guild_id) DO UPDATE SET log_channel_id = excluded.log_channel_id",
    )
    .bind(guild_id as i64)
    .bind(channel_id as i64)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn disable_logging(pool: &SqlitePool, guild_id: u64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE mod_config SET log_channel_id = NULL WHERE guild_id = ?")
        .bind(guild_id as i64)
        .execute(pool)
        .await?;

    Ok(())
}

impl From<(i64, Option<i64>, Option<i64>)> for ModConfig {
    fn from((guild_id, auto_role_id, log_channel_id): (i64, Option<i64>, Option<i64>)) -> Self {
        Self {
            guild_id,
            auto_role_id,
            log_channel_id,
        }
    }
}
