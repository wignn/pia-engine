use sqlx::postgres::PgPool;

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = atlsd_common::db::create_pool(database_url).await?;
    sqlx::migrate!("../../db/migrations/core")
        .run(&pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "migration failed");
            sqlx::Error::Configuration(e.into())
        })?;

    tracing::info!("database migrations applied");
    Ok(pool)
}
