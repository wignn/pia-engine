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

#[derive(Debug, Clone)]
pub struct ModConfig {
    pub guild_id: i64,
    pub auto_role_id: Option<i64>,
    pub log_channel_id: Option<i64>,
}

pub struct ModerationRepository;

impl ModerationRepository {
    pub async fn add_warning(
        pool: &SqlitePool, guild_id: u64, user_id: u64, moderator_id: u64, reason: &str,
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

    pub async fn get_warnings(pool: &SqlitePool, guild_id: u64, user_id: u64) -> Result<Vec<Warning>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (i64, i64, i64, i64, String, String)>(
            "SELECT id, guild_id, user_id, moderator_id, reason, created_at FROM mod_warnings WHERE guild_id = ? AND user_id = ? ORDER BY created_at DESC",
        )
        .bind(guild_id as i64)
        .bind(user_id as i64)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(|(id, guild_id, user_id, moderator_id, reason, created_at)| Warning {
            id, guild_id, user_id, moderator_id, reason, created_at,
        }).collect())
    }

    pub async fn get_warning_count(pool: &SqlitePool, guild_id: u64, user_id: u64) -> Result<i64, sqlx::Error> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM mod_warnings WHERE guild_id = ? AND user_id = ?",
        )
        .bind(guild_id as i64)
        .bind(user_id as i64)
        .fetch_one(pool)
        .await?;
        Ok(count.0)
    }

    pub async fn clear_warnings(pool: &SqlitePool, guild_id: u64, user_id: u64) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM mod_warnings WHERE guild_id = ? AND user_id = ?")
            .bind(guild_id as i64)
            .bind(user_id as i64)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn delete_warning(pool: &SqlitePool, warning_id: i64, guild_id: u64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM mod_warnings WHERE id = ? AND guild_id = ?")
            .bind(warning_id)
            .bind(guild_id as i64)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn get_config(pool: &SqlitePool, guild_id: u64) -> Result<Option<ModConfig>, sqlx::Error> {
        let row = sqlx::query_as::<_, (i64, Option<i64>, Option<i64>)>(
            "SELECT guild_id, auto_role_id, log_channel_id FROM mod_config WHERE guild_id = ?",
        )
        .bind(guild_id as i64)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|(guild_id, auto_role_id, log_channel_id)| ModConfig {
            guild_id, auto_role_id, log_channel_id,
        }))
    }

    pub async fn set_auto_role(pool: &SqlitePool, guild_id: u64, role_id: u64) -> Result<(), sqlx::Error> {
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

    pub async fn set_log_channel(pool: &SqlitePool, guild_id: u64, channel_id: u64) -> Result<(), sqlx::Error> {
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
}
