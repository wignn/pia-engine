use super::{feed_channel, sent_item};
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct ForexChannel {
    pub id: i64,
    pub channel_id: i64,
    pub guild_id: i64,
    pub is_active: bool,
}

pub struct ForexRepository;

impl ForexRepository {
    pub async fn insert_channel(
        pool: &SqlitePool,
        guild_id: u64,
        channel_id: u64,
    ) -> Result<(), sqlx::Error> {
        feed_channel::upsert_by_guild(pool, "forex_channels", guild_id, channel_id).await
    }

    pub async fn disable_channel(pool: &SqlitePool, guild_id: u64) -> Result<(), sqlx::Error> {
        feed_channel::set_active_by_guild(pool, "forex_channels", guild_id, false).await
    }

    pub async fn enable_channel(pool: &SqlitePool, guild_id: u64) -> Result<(), sqlx::Error> {
        feed_channel::set_active_by_guild(pool, "forex_channels", guild_id, true).await
    }

    pub async fn get_active_channels(pool: &SqlitePool) -> Result<Vec<ForexChannel>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (i64, i64, i64, bool)>(
            "SELECT id, channel_id, guild_id, is_active FROM forex_channels WHERE is_active = 1",
        )
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(id, channel_id, guild_id, is_active)| ForexChannel {
                id,
                channel_id,
                guild_id,
                is_active,
            })
            .collect())
    }

    pub async fn get_channel(
        pool: &SqlitePool,
        guild_id: u64,
    ) -> Result<Option<ForexChannel>, sqlx::Error> {
        let row = sqlx::query_as::<_, (i64, i64, i64, bool)>(
            "SELECT id, channel_id, guild_id, is_active FROM forex_channels WHERE guild_id = ?",
        )
        .bind(guild_id as i64)
        .fetch_optional(pool)
        .await?;

        Ok(
            row.map(|(id, channel_id, guild_id, is_active)| ForexChannel {
                id,
                channel_id,
                guild_id,
                is_active,
            }),
        )
    }

    pub async fn is_news_sent(pool: &SqlitePool, news_id: &str) -> Result<bool, sqlx::Error> {
        sent_item::exists_with_type(pool, news_id, "news").await
    }

    pub async fn insert_news(
        pool: &SqlitePool,
        news_id: &str,
        source: &str,
    ) -> Result<(), sqlx::Error> {
        sent_item::insert(pool, news_id, "news", source).await
    }

    pub async fn cleanup_old_news(pool: &SqlitePool, days: i64) -> Result<u64, sqlx::Error> {
        sent_item::cleanup(pool, "news", days).await
    }
}
