use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;
use tracing::{error, info};
use url::Url;

/// Create a PostgreSQL connection pool and run embedded migrations.
pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    if let Ok(parsed) = Url::parse(database_url) {
        info!(
            db_host = parsed.host_str().unwrap_or("unknown"),
            db_port = parsed.port_or_known_default().unwrap_or(5432),
            db_name = parsed.path().trim_start_matches('/'),
            "connecting to postgres"
        );
    }

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .min_connections(0)
        .max_lifetime(Duration::from_secs(30 * 60))
        .idle_timeout(Duration::from_secs(5 * 60))
        .acquire_timeout(Duration::from_secs(30))
        .test_before_acquire(true)
        .connect(database_url)
        .await?;

    sqlx::query("SELECT 1").execute(&pool).await.map_err(|e| {
        error!(error = %e, "database health check failed");
        e
    })?;

    info!(max_conns = 10, "database connected");

    sqlx::migrate!("./migrations").run(&pool).await.map_err(|e| {
        tracing::error!(error = %e, "migration failed");
        sqlx::Error::Configuration(e.into())
    })?;

    info!("database migrations applied");
    Ok(pool)
}
