use sqlx::PgPool;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct NewsSource {
    pub id: String,
    pub name: String,
    pub rss_url: Option<String>,
    pub poll_interval_sec: i32,
}

impl NewsSource {
    pub fn poll_interval(&self) -> i32 {
        self.poll_interval_sec.max(15)
    }

    pub fn retry_interval(&self) -> i32 {
        self.poll_interval_sec.max(60)
    }
}

pub async fn due_sources(pool: &PgPool) -> anyhow::Result<Vec<NewsSource>> {
    let sources = sqlx::query_as::<_, NewsSource>(
        "SELECT id, name, rss_url, poll_interval_sec FROM news.forex_news_sources WHERE is_active = TRUE AND source_type = 'rss' AND rss_url IS NOT NULL AND (next_allowed_poll_at IS NULL OR next_allowed_poll_at <= NOW()) ORDER BY priority ASC, name ASC",
    )
    .fetch_all(pool)
    .await?;

    Ok(sources)
}

pub async fn record_success(
    pool: &PgPool,
    source: &NewsSource,
    status: i32,
    latency_ms: i64,
) -> anyhow::Result<()> {
    sqlx::query(
        "UPDATE news.forex_news_sources SET last_success_at = NOW(), last_status = $2, last_latency_ms = $3, success_count = success_count + 1, last_error_message = NULL, next_allowed_poll_at = NOW() + make_interval(secs => $4), updated_at = NOW() WHERE id = $1",
    )
    .bind(&source.id)
    .bind(status)
    .bind(latency_ms)
    .bind(source.poll_interval())
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn record_error(
    pool: &PgPool,
    source: &NewsSource,
    status: i32,
    latency_ms: i64,
    message: &str,
) -> anyhow::Result<()> {
    sqlx::query(
        "UPDATE news.forex_news_sources SET last_error_at = NOW(), last_status = $2, last_latency_ms = $3, error_count = error_count + 1, last_error_message = $4, next_allowed_poll_at = NOW() + make_interval(secs => $5), updated_at = NOW() WHERE id = $1",
    )
    .bind(&source.id)
    .bind(status)
    .bind(latency_ms)
    .bind(message)
    .bind(source.retry_interval())
    .execute(pool)
    .await?;

    Ok(())
}
