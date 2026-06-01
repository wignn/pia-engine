use super::feed_channel;
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct VolatilityChannel {
    pub id: i64,
    pub channel_id: i64,
    pub guild_id: i64,
    pub is_active: bool,
}

pub struct VolatilityRepository;

impl VolatilityRepository {
    pub async fn insert_channel(
        pool: &SqlitePool,
        guild_id: u64,
        channel_id: u64,
    ) -> Result<(), sqlx::Error> {
        feed_channel::upsert_by_guild(pool, "volatility_channels", guild_id, channel_id).await
    }

    pub async fn disable_channel(pool: &SqlitePool, guild_id: u64) -> Result<(), sqlx::Error> {
        feed_channel::set_active_by_guild(pool, "volatility_channels", guild_id, false).await
    }

    pub async fn get_active_channels(
        pool: &SqlitePool,
    ) -> Result<Vec<VolatilityChannel>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (i64, i64, i64, bool)>(
            "SELECT id, channel_id, guild_id, is_active FROM volatility_channels WHERE is_active = 1",
        )
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(id, channel_id, guild_id, is_active)| VolatilityChannel {
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
    ) -> Result<Option<VolatilityChannel>, sqlx::Error> {
        let row = sqlx::query_as::<_, (i64, i64, i64, bool)>(
            "SELECT id, channel_id, guild_id, is_active FROM volatility_channels WHERE guild_id = ?",
        )
        .bind(guild_id as i64)
        .fetch_optional(pool)
        .await?;

        Ok(
            row.map(|(id, channel_id, guild_id, is_active)| VolatilityChannel {
                id,
                channel_id,
                guild_id,
                is_active,
            }),
        )
    }
}
