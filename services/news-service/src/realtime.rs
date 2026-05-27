use atlsd_eventbus::{subjects, EventBusMode};
use serde_json::{json, Value};
use tracing::{error, info, warn};

use crate::config::Config;

pub async fn run(cfg: Config, pool: sqlx::PgPool) {
    if !matches!(
        EventBusMode::from_env_value(&cfg.eventbus_mode),
        EventBusMode::Nats | EventBusMode::Dual
    ) {
        return;
    }

    loop {
        match async_nats::connect(&cfg.nats_url).await {
            Ok(client) => {
                info!(url = %cfg.nats_url, "news-service realtime publisher connected to NATS");
                publish_loop(&cfg, &pool, client).await;
            }
            Err(err) => {
                error!(error = %err, url = %cfg.nats_url, "news-service NATS connection failed");
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
    }
}

async fn publish_loop(cfg: &Config, pool: &sqlx::PgPool, client: async_nats::Client) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(cfg.realtime_poll_sec));
    let mut last_forex: Option<chrono::DateTime<chrono::Utc>> = None;
    let mut last_stock: Option<chrono::DateTime<chrono::Utc>> = None;

    loop {
        interval.tick().await;
        match fetch_forex(pool, last_forex).await {
            Ok(rows) => {
                for (seen_at, article) in rows.into_iter().rev() {
                    publish(&client, subjects::NEWS_FOREX_PROCESSED_V1, &article).await;
                    last_forex = Some(last_forex.map_or(seen_at, |current| current.max(seen_at)));
                }
            }
            Err(err) => warn!(error = %err, "failed to poll realtime forex news"),
        }

        match fetch_stock(pool, last_stock).await {
            Ok(rows) => {
                for (seen_at, article) in rows.into_iter().rev() {
                    publish(&client, subjects::NEWS_STOCK_PROCESSED_V1, &article).await;
                    last_stock = Some(last_stock.map_or(seen_at, |current| current.max(seen_at)));
                }
            }
            Err(err) => warn!(error = %err, "failed to poll realtime stock news"),
        }
    }
}

async fn publish(client: &async_nats::Client, subject: &str, article: &Value) {
    let payload = json!({ "article": article }).to_string();
    if let Err(err) = client.publish(subject.to_string(), payload.into()).await {
        warn!(error = %err, subject, "failed to publish realtime news event");
    }
}

async fn fetch_forex(
    pool: &sqlx::PgPool,
    after: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<Vec<(chrono::DateTime<chrono::Utc>, Value)>, sqlx::Error> {
    let after = after.unwrap_or_else(|| chrono::Utc::now() - chrono::Duration::seconds(30));
    let rows = sqlx::query_as::<_, (
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<chrono::DateTime<chrono::Utc>>,
        Option<chrono::DateTime<chrono::Utc>>,
        chrono::DateTime<chrono::Utc>,
    )>(
        "SELECT a.id::text, a.original_title, a.summary, COALESCE(s.name, 'Unknown') AS source_name, a.original_url, an.sentiment, an.impact_level, a.published_at, a.processed_at, COALESCE(a.processed_at, a.published_at, a.created_at) AS seen_at FROM news.forex_news_articles a LEFT JOIN news.forex_news_sources s ON a.source_id = s.id LEFT JOIN news.forex_news_analyses an ON a.id = an.article_id WHERE a.is_processed = TRUE AND COALESCE(a.processed_at, a.published_at, a.created_at) > $1 ORDER BY seen_at DESC LIMIT 50",
    )
    .bind(after)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            (
                row.9,
                json!({
                    "id": row.0,
                    "title": row.1,
                    "original_title": row.1,
                    "summary": row.2,
                    "source_name": row.3,
                    "url": row.4,
                    "original_url": row.4,
                    "sentiment": row.5,
                    "impact_level": row.6,
                    "published_at": row.7,
                    "processed_at": row.8,
                }),
            )
        })
        .collect())
}

async fn fetch_stock(
    pool: &sqlx::PgPool,
    after: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<Vec<(chrono::DateTime<chrono::Utc>, Value)>, sqlx::Error> {
    let after = after.unwrap_or_else(|| chrono::Utc::now() - chrono::Duration::seconds(30));
    let rows = sqlx::query_as::<_, (
        String,
        String,
        Option<String>,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<chrono::DateTime<chrono::Utc>>,
    )>(
        "SELECT content_hash, title, summary, source_name, category, tickers, sentiment, impact_level, processed_at FROM news.stock_news WHERE is_processed = TRUE AND processed_at > $1 ORDER BY processed_at DESC LIMIT 50",
    )
    .bind(after)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .filter_map(|row| {
            row.8.map(|seen_at| {
                (
                    seen_at,
                    json!({
                        "id": row.0,
                        "content_hash": row.0,
                        "title": row.1,
                        "summary": row.2,
                        "source_name": row.3,
                        "category": row.4,
                        "tickers": row.5,
                        "sentiment": row.6,
                        "impact_level": row.7,
                        "published_at": row.8,
                        "processed_at": row.8,
                    }),
                )
            })
        })
        .collect())
}
