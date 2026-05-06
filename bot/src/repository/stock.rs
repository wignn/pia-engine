use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct StockChannel {
    pub id: i64,
    pub channel_id: i64,
    pub guild_id: i64,
    pub tickers_filter: Option<String>,
    pub min_impact: Option<String>,
    pub categories: Option<String>,
    pub mention_everyone: bool,
    pub is_active: bool,
}

pub struct StockRepository;

impl StockRepository {
    pub async fn insert_channel(
        pool: &SqlitePool,
        guild_id: u64,
        channel_id: u64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO stock_channels (guild_id, channel_id, is_active)
             VALUES (?, ?, 1)
             ON CONFLICT(channel_id) DO UPDATE SET guild_id = excluded.guild_id, is_active = 1",
        )
        .bind(guild_id as i64)
        .bind(channel_id as i64)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn disable_channel(pool: &SqlitePool, channel_id: u64) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE stock_channels SET is_active = 0 WHERE channel_id = ?")
            .bind(channel_id as i64)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn get_active_channels(
        pool: &SqlitePool,
    ) -> Result<Vec<StockChannel>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (i64, i64, i64, Option<String>, Option<String>, Option<String>, bool, bool)>(
            "SELECT id, channel_id, guild_id, tickers_filter, min_impact, categories, mention_everyone, is_active
             FROM stock_channels WHERE is_active = 1",
        )
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(id, channel_id, guild_id, tickers_filter, min_impact, categories, mention_everyone, is_active)| {
                    StockChannel {
                        id,
                        channel_id,
                        guild_id,
                        tickers_filter,
                        min_impact,
                        categories,
                        mention_everyone,
                        is_active,
                    }
                },
            )
            .collect())
    }

    pub async fn get_channel(
        pool: &SqlitePool,
        channel_id: u64,
    ) -> Result<Option<StockChannel>, sqlx::Error> {
        let row = sqlx::query_as::<_, (i64, i64, i64, Option<String>, Option<String>, Option<String>, bool, bool)>(
            "SELECT id, channel_id, guild_id, tickers_filter, min_impact, categories, mention_everyone, is_active
             FROM stock_channels WHERE channel_id = ?",
        )
        .bind(channel_id as i64)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(
            |(id, channel_id, guild_id, tickers_filter, min_impact, categories, mention_everyone, is_active)| {
                StockChannel {
                    id,
                    channel_id,
                    guild_id,
                    tickers_filter,
                    min_impact,
                    categories,
                    mention_everyone,
                    is_active,
                }
            },
        ))
    }

    pub async fn is_stock_news_sent(
        pool: &SqlitePool,
        news_id: &str,
    ) -> Result<bool, sqlx::Error> {
        let prefixed_id = format!("stock_{}", news_id);
        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM sent_items WHERE item_id = ?")
                .bind(&prefixed_id)
                .fetch_one(pool)
                .await?;

        Ok(count.0 > 0)
    }

    pub async fn insert_stock_news(
        pool: &SqlitePool,
        news_id: &str,
        source: &str,
    ) -> Result<(), sqlx::Error> {
        let prefixed_id = format!("stock_{}", news_id);
        let now = chrono::Utc::now().timestamp();
        sqlx::query(
            "INSERT INTO sent_items (item_id, item_type, source, sent_at) VALUES (?, 'stock', ?, ?) ON CONFLICT(item_id) DO NOTHING",
        )
        .bind(&prefixed_id)
        .bind(source)
        .bind(now)
        .execute(pool)
        .await?;

        Ok(())
    }
}
