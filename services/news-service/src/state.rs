use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub metrics: std::sync::Arc<atlsd_observability::MetricsRegistry>,
}
