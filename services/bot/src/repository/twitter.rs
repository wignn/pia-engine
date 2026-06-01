use super::{feed_channel, sent_item};
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct TwitterChannel {
    pub id: i64,
    pub channel_id: i64,
    pub guild_id: i64,
    pub is_active: bool,
}

pub struct TwitterRepository;

impl TwitterRepository {
    pub async fn insert_channel(
        pool: &SqlitePool,
        guild_id: u64,
        channel_id: u64,
    ) -> Result<(), sqlx::Error> {
        feed_channel::upsert_by_guild(pool, "twitter_channels", guild_id, channel_id).await
    }

    pub async fn disable_channel(pool: &SqlitePool, guild_id: u64) -> Result<(), sqlx::Error> {
        feed_channel::set_active_by_guild(pool, "twitter_channels", guild_id, false).await
    }

    pub async fn enable_channel(pool: &SqlitePool, guild_id: u64) -> Result<(), sqlx::Error> {
        feed_channel::set_active_by_guild(pool, "twitter_channels", guild_id, true).await
    }

    pub async fn get_active_channels(
        pool: &SqlitePool,
    ) -> Result<Vec<TwitterChannel>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (i64, i64, i64, bool)>(
            "SELECT id, channel_id, guild_id, is_active FROM twitter_channels WHERE is_active = 1",
        )
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(id, channel_id, guild_id, is_active)| TwitterChannel {
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
    ) -> Result<Option<TwitterChannel>, sqlx::Error> {
        let row = sqlx::query_as::<_, (i64, i64, i64, bool)>(
            "SELECT id, channel_id, guild_id, is_active FROM twitter_channels WHERE guild_id = ?",
        )
        .bind(guild_id as i64)
        .fetch_optional(pool)
        .await?;

        Ok(
            row.map(|(id, channel_id, guild_id, is_active)| TwitterChannel {
                id,
                channel_id,
                guild_id,
                is_active,
            }),
        )
    }

    pub async fn is_tweet_sent(pool: &SqlitePool, tweet_id: &str) -> Result<bool, sqlx::Error> {
        let prefixed_id = format!("tweet_{}", tweet_id);
        sent_item::exists(pool, &prefixed_id).await
    }

    pub async fn insert_tweet(
        pool: &SqlitePool,
        tweet_id: &str,
        author: &str,
    ) -> Result<(), sqlx::Error> {
        let prefixed_id = format!("tweet_{}", tweet_id);
        sent_item::insert(pool, &prefixed_id, "tweet", author).await
    }

    pub async fn cleanup_old_tweets(pool: &SqlitePool, days: i64) -> Result<u64, sqlx::Error> {
        sent_item::cleanup(pool, "tweet", days).await
    }
}
