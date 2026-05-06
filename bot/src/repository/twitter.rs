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
        sqlx::query(
            "INSERT INTO twitter_channels (guild_id, channel_id, is_active)
             VALUES (?, ?, 1)
             ON CONFLICT(guild_id) DO UPDATE SET channel_id = excluded.channel_id, is_active = 1",
        )
        .bind(guild_id as i64)
        .bind(channel_id as i64)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn disable_channel(pool: &SqlitePool, guild_id: u64) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE twitter_channels SET is_active = 0 WHERE guild_id = ?")
            .bind(guild_id as i64)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn enable_channel(pool: &SqlitePool, guild_id: u64) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE twitter_channels SET is_active = 1 WHERE guild_id = ?")
            .bind(guild_id as i64)
            .execute(pool)
            .await?;

        Ok(())
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

        Ok(row.map(|(id, channel_id, guild_id, is_active)| TwitterChannel {
            id,
            channel_id,
            guild_id,
            is_active,
        }))
    }

    pub async fn is_tweet_sent(pool: &SqlitePool, tweet_id: &str) -> Result<bool, sqlx::Error> {
        let prefixed_id = format!("tweet_{}", tweet_id);
        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM sent_items WHERE item_id = ?")
                .bind(&prefixed_id)
                .fetch_one(pool)
                .await?;

        Ok(count.0 > 0)
    }

    pub async fn insert_tweet(
        pool: &SqlitePool,
        tweet_id: &str,
        author: &str,
    ) -> Result<(), sqlx::Error> {
        let prefixed_id = format!("tweet_{}", tweet_id);
        let now = chrono::Utc::now().timestamp();
        sqlx::query(
            "INSERT INTO sent_items (item_id, item_type, source, sent_at) VALUES (?, 'tweet', ?, ?) ON CONFLICT(item_id) DO NOTHING",
        )
        .bind(&prefixed_id)
        .bind(author)
        .bind(now)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn cleanup_old_tweets(pool: &SqlitePool, days: i64) -> Result<u64, sqlx::Error> {
        let cutoff = chrono::Utc::now().timestamp() - (days * 86400);
        let result =
            sqlx::query("DELETE FROM sent_items WHERE item_type = 'tweet' AND sent_at < ?")
                .bind(cutoff)
                .execute(pool)
                .await?;

        Ok(result.rows_affected())
    }
}
