use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct PriceAlert {
    pub id: i64,
    pub user_id: i64,
    pub guild_id: i64,
    pub symbol: String,
    pub target_price: f64,
    pub direction: String,
    pub is_triggered: bool,
    pub created_at: String,
    pub triggered_at: Option<String>,
}

pub struct PriceAlertRepository;

impl PriceAlertRepository {
    pub async fn create_alert(
        pool: &SqlitePool, user_id: u64, guild_id: u64, symbol: &str, target_price: f64, direction: &str,
    ) -> Result<PriceAlert, sqlx::Error> {
        let result = sqlx::query(
            "INSERT INTO price_alerts (user_id, guild_id, symbol, target_price, direction) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(user_id as i64)
        .bind(guild_id as i64)
        .bind(symbol)
        .bind(target_price)
        .bind(direction)
        .execute(pool)
        .await?;

        let id = result.last_insert_rowid();
        let row = sqlx::query_as::<_, (i64, i64, i64, String, f64, String, bool, String, Option<String>)>(
            "SELECT id, user_id, guild_id, symbol, target_price, direction, is_triggered, created_at, triggered_at FROM price_alerts WHERE id = ?",
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(PriceAlert {
            id: row.0, user_id: row.1, guild_id: row.2, symbol: row.3,
            target_price: row.4, direction: row.5, is_triggered: row.6,
            created_at: row.7, triggered_at: row.8,
        })
    }

    pub async fn get_user_alerts(pool: &SqlitePool, user_id: u64) -> Result<Vec<PriceAlert>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (i64, i64, i64, String, f64, String, bool, String, Option<String>)>(
            "SELECT id, user_id, guild_id, symbol, target_price, direction, is_triggered, created_at, triggered_at
             FROM price_alerts WHERE user_id = ? AND is_triggered = 0 ORDER BY created_at DESC",
        )
        .bind(user_id as i64)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(|r| PriceAlert {
            id: r.0, user_id: r.1, guild_id: r.2, symbol: r.3,
            target_price: r.4, direction: r.5, is_triggered: r.6,
            created_at: r.7, triggered_at: r.8,
        }).collect())
    }

    pub async fn get_active_alerts_by_symbol(pool: &SqlitePool, symbol: &str) -> Result<Vec<PriceAlert>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (i64, i64, i64, String, f64, String, bool, String, Option<String>)>(
            "SELECT id, user_id, guild_id, symbol, target_price, direction, is_triggered, created_at, triggered_at
             FROM price_alerts WHERE symbol = ? AND is_triggered = 0",
        )
        .bind(symbol)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(|r| PriceAlert {
            id: r.0, user_id: r.1, guild_id: r.2, symbol: r.3,
            target_price: r.4, direction: r.5, is_triggered: r.6,
            created_at: r.7, triggered_at: r.8,
        }).collect())
    }

    pub async fn get_all_active_symbols(pool: &SqlitePool) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (String,)>(
            "SELECT DISTINCT symbol FROM price_alerts WHERE is_triggered = 0",
        )
        .fetch_all(pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    pub async fn trigger_alert(pool: &SqlitePool, alert_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE price_alerts SET is_triggered = 1, triggered_at = datetime('now') WHERE id = ?",
        )
        .bind(alert_id)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn delete_alert(pool: &SqlitePool, alert_id: i64, user_id: u64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM price_alerts WHERE id = ? AND user_id = ? AND is_triggered = 0",
        )
        .bind(alert_id)
        .bind(user_id as i64)
        .execute(pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn count_user_alerts(pool: &SqlitePool, user_id: u64) -> Result<i64, sqlx::Error> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM price_alerts WHERE user_id = ? AND is_triggered = 0",
        )
        .bind(user_id as i64)
        .fetch_one(pool)
        .await?;
        Ok(count.0)
    }
}
