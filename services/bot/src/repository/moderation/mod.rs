mod config;
mod warnings;

pub use config::ModConfig;
pub use warnings::Warning;

use sqlx::SqlitePool;

pub struct ModerationRepository;

impl ModerationRepository {
    pub async fn add_warning(
        pool: &SqlitePool,
        guild_id: u64,
        user_id: u64,
        moderator_id: u64,
        reason: &str,
    ) -> Result<i64, sqlx::Error> {
        warnings::add(pool, guild_id, user_id, moderator_id, reason).await
    }

    pub async fn get_warnings(
        pool: &SqlitePool,
        guild_id: u64,
        user_id: u64,
    ) -> Result<Vec<Warning>, sqlx::Error> {
        warnings::list(pool, guild_id, user_id).await
    }

    pub async fn get_warning_count(
        pool: &SqlitePool,
        guild_id: u64,
        user_id: u64,
    ) -> Result<i64, sqlx::Error> {
        warnings::count(pool, guild_id, user_id).await
    }

    pub async fn clear_warnings(
        pool: &SqlitePool,
        guild_id: u64,
        user_id: u64,
    ) -> Result<u64, sqlx::Error> {
        warnings::clear(pool, guild_id, user_id).await
    }

    pub async fn delete_warning(
        pool: &SqlitePool,
        warning_id: i64,
        guild_id: u64,
    ) -> Result<bool, sqlx::Error> {
        warnings::delete(pool, warning_id, guild_id).await
    }

    pub async fn get_config(
        pool: &SqlitePool,
        guild_id: u64,
    ) -> Result<Option<ModConfig>, sqlx::Error> {
        config::get(pool, guild_id).await
    }

    pub async fn set_auto_role(
        pool: &SqlitePool,
        guild_id: u64,
        role_id: u64,
    ) -> Result<(), sqlx::Error> {
        config::set_auto_role(pool, guild_id, role_id).await
    }

    pub async fn disable_auto_role(pool: &SqlitePool, guild_id: u64) -> Result<(), sqlx::Error> {
        config::disable_auto_role(pool, guild_id).await
    }

    pub async fn set_log_channel(
        pool: &SqlitePool,
        guild_id: u64,
        channel_id: u64,
    ) -> Result<(), sqlx::Error> {
        config::set_log_channel(pool, guild_id, channel_id).await
    }

    pub async fn disable_logging(pool: &SqlitePool, guild_id: u64) -> Result<(), sqlx::Error> {
        config::disable_logging(pool, guild_id).await
    }
}
