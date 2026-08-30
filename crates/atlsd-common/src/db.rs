use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use std::str::FromStr;
use std::time::Duration;
use tracing::{error, info};
use url::Url;

/// Creates a standard PostgreSQL connection pool with default settings (10 max connections).
pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    create_resilient_pool(database_url, 10, 0).await
}

/// Creates a tuned, resilient PostgreSQL connection pool.
/// Automatically detects PgBouncer transaction pooling (port 6432) and disables
/// SQLx prepared statement caching to prevent 'prepared statement already exists' conflicts.
pub async fn create_resilient_pool(
    database_url: &str,
    max_conns: u32,
    min_conns: u32,
) -> Result<PgPool, sqlx::Error> {
    if let Ok(parsed) = Url::parse(database_url) {
        info!(
            db_host = parsed.host_str().unwrap_or("unknown"),
            db_port = parsed.port_or_known_default().unwrap_or(5432),
            db_name = parsed.path().trim_start_matches('/'),
            max_conns,
            min_conns,
            "connecting to postgres / pgbouncer"
        );
    }

    let is_pgbouncer = database_url.contains(":6432") || database_url.contains("pgbouncer");
    let mut connect_opts = PgConnectOptions::from_str(database_url)?;

    let app_name = std::env::var("SERVICE_NAME").unwrap_or_else(|_| "atlsd-service".to_string());
    connect_opts = connect_opts.application_name(&app_name);

    if is_pgbouncer {
        info!("pgbouncer transaction pool detected; disabling statement cache");
        connect_opts = connect_opts.statement_cache_capacity(0);
    }

    let pool = PgPoolOptions::new()
        .max_connections(max_conns)
        .min_connections(min_conns)
        .max_lifetime(Duration::from_secs(15 * 60))
        .idle_timeout(Duration::from_secs(5 * 60))
        .acquire_timeout(Duration::from_secs(10))
        .test_before_acquire(true)
        .connect_with(connect_opts)
        .await?;

    sqlx::query("SELECT 1").execute(&pool).await.map_err(|e| {
        error!(error = %e, "database health check failed");
        e
    })?;

    info!(max_conns, min_conns, is_pgbouncer, "database pool initialized and verified");
    Ok(pool)
}
