mod model;
mod queries;

pub use model::PriceAlert;
use model::PriceAlertRow;
use sqlx::SqlitePool;

pub struct PriceAlertRepository;

impl PriceAlertRepository {
    pub async fn create_alert(
        pool: &SqlitePool,
        user_id: u64,
        guild_id: u64,
        symbol: &str,
        target_price: f64,
        direction: &str,
    ) -> Result<PriceAlert, sqlx::Error> {
        queries::create(pool, user_id, guild_id, symbol, target_price, direction).await
    }

    pub async fn get_user_alerts(
        pool: &SqlitePool,
        user_id: u64,
    ) -> Result<Vec<PriceAlert>, sqlx::Error> {
        queries::get_user_active(pool, user_id).await
    }

    pub async fn get_active_alerts_by_symbol(
        pool: &SqlitePool,
        symbol: &str,
    ) -> Result<Vec<PriceAlert>, sqlx::Error> {
        queries::get_active_by_symbol(pool, symbol).await
    }

    pub async fn get_all_active_symbols(pool: &SqlitePool) -> Result<Vec<String>, sqlx::Error> {
        queries::get_all_active_symbols(pool).await
    }

    pub async fn trigger_alert(pool: &SqlitePool, alert_id: i64) -> Result<(), sqlx::Error> {
        queries::trigger(pool, alert_id).await
    }

    pub async fn delete_alert(
        pool: &SqlitePool,
        alert_id: i64,
        user_id: u64,
    ) -> Result<bool, sqlx::Error> {
        queries::delete(pool, alert_id, user_id).await
    }

    pub async fn count_user_alerts(pool: &SqlitePool, user_id: u64) -> Result<i64, sqlx::Error> {
        queries::count_user_active(pool, user_id).await
    }
}
