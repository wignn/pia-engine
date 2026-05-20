use sqlx::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::Arc;

pub type DbPool = Arc<SqlitePool>;

pub async fn create_pool(db_path: &str) -> Result<DbPool, sqlx::Error> {
    let url = format!("sqlite:{}?mode=rwc", db_path);
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await?;

    sqlx::migrate!("../../db/migrations/bot").run(&pool).await?;

    println!("[OK] SQLite database ready ({})", db_path);
    Ok(Arc::new(pool))
}
