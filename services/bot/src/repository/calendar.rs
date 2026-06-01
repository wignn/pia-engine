use super::sent_item;
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct CalendarChannel {
    pub id: i64,
    pub channel_id: i64,
    pub guild_id: i64,
    pub is_active: bool,
    pub mention_everyone: bool,
}

pub struct CalendarRepository;

impl CalendarRepository {
    pub async fn insert_channel(
        pool: &SqlitePool,
        guild_id: u64,
        channel_id: u64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO calendar_channels (guild_id, channel_id, is_active, mention_everyone)
             VALUES (?, ?, 1, 0)
             ON CONFLICT(guild_id) DO UPDATE SET channel_id = excluded.channel_id, is_active = 1",
        )
        .bind(guild_id as i64)
        .bind(channel_id as i64)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn disable_channel(pool: &SqlitePool, guild_id: u64) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE calendar_channels SET is_active = 0 WHERE guild_id = ?")
            .bind(guild_id as i64)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn enable_channel(pool: &SqlitePool, guild_id: u64) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE calendar_channels SET is_active = 1 WHERE guild_id = ?")
            .bind(guild_id as i64)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn set_mention_everyone(
        pool: &SqlitePool,
        guild_id: u64,
        mention: bool,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE calendar_channels SET mention_everyone = ? WHERE guild_id = ?")
            .bind(mention)
            .bind(guild_id as i64)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn get_active_channels(
        pool: &SqlitePool,
    ) -> Result<Vec<CalendarChannel>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (i64, i64, i64, bool, bool)>(
            "SELECT id, channel_id, guild_id, is_active, mention_everyone FROM calendar_channels WHERE is_active = 1",
        )
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(id, channel_id, guild_id, is_active, mention_everyone)| CalendarChannel {
                    id,
                    channel_id,
                    guild_id,
                    is_active,
                    mention_everyone,
                },
            )
            .collect())
    }

    pub async fn get_channel(
        pool: &SqlitePool,
        guild_id: u64,
    ) -> Result<Option<CalendarChannel>, sqlx::Error> {
        let row = sqlx::query_as::<_, (i64, i64, i64, bool, bool)>(
            "SELECT id, channel_id, guild_id, is_active, mention_everyone FROM calendar_channels WHERE guild_id = ?",
        )
        .bind(guild_id as i64)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(
            |(id, channel_id, guild_id, is_active, mention_everyone)| CalendarChannel {
                id,
                channel_id,
                guild_id,
                is_active,
                mention_everyone,
            },
        ))
    }

    pub async fn is_event_sent(pool: &SqlitePool, event_id: &str) -> Result<bool, sqlx::Error> {
        let prefixed_id = format!("cal_{}", event_id);
        sent_item::exists(pool, &prefixed_id).await
    }

    pub async fn insert_event(
        pool: &SqlitePool,
        event_id: &str,
        event_title: &str,
    ) -> Result<(), sqlx::Error> {
        let prefixed_id = format!("cal_{}", event_id);
        sent_item::insert(pool, &prefixed_id, "calendar", event_title).await
    }

    pub async fn cleanup_old_events(pool: &SqlitePool, days: i64) -> Result<u64, sqlx::Error> {
        sent_item::cleanup(pool, "calendar", days).await
    }
}
