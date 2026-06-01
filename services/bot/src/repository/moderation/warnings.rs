use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct Warning {
    pub id: i64,
    pub guild_id: i64,
    pub user_id: i64,
    pub moderator_id: i64,
    pub reason: String,
    pub created_at: String,
}

pub async fn add(
    pool: &SqlitePool,
    guild_id: u64,
    user_id: u64,
    moderator_id: u64,
    reason: &str,
) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO mod_warnings (guild_id, user_id, moderator_id, reason) VALUES (?, ?, ?, ?)",
    )
    .bind(guild_id as i64)
    .bind(user_id as i64)
    .bind(moderator_id as i64)
    .bind(reason)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

pub async fn list(
    pool: &SqlitePool,
    guild_id: u64,
    user_id: u64,
) -> Result<Vec<Warning>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (i64, i64, i64, i64, String, String)>(
        "SELECT id, guild_id, user_id, moderator_id, reason, created_at FROM mod_warnings WHERE guild_id = ? AND user_id = ? ORDER BY created_at DESC",
    )
    .bind(guild_id as i64)
    .bind(user_id as i64)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(Warning::from).collect())
}

pub async fn count(pool: &SqlitePool, guild_id: u64, user_id: u64) -> Result<i64, sqlx::Error> {
    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM mod_warnings WHERE guild_id = ? AND user_id = ?")
            .bind(guild_id as i64)
            .bind(user_id as i64)
            .fetch_one(pool)
            .await?;

    Ok(count.0)
}

pub async fn clear(pool: &SqlitePool, guild_id: u64, user_id: u64) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM mod_warnings WHERE guild_id = ? AND user_id = ?")
        .bind(guild_id as i64)
        .bind(user_id as i64)
        .execute(pool)
        .await?;

    Ok(result.rows_affected())
}

pub async fn delete(
    pool: &SqlitePool,
    warning_id: i64,
    guild_id: u64,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM mod_warnings WHERE id = ? AND guild_id = ?")
        .bind(warning_id)
        .bind(guild_id as i64)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

impl From<(i64, i64, i64, i64, String, String)> for Warning {
    fn from(
        (id, guild_id, user_id, moderator_id, reason, created_at): (
            i64,
            i64,
            i64,
            i64,
            String,
            String,
        ),
    ) -> Self {
        Self {
            id,
            guild_id,
            user_id,
            moderator_id,
            reason,
            created_at,
        }
    }
}
