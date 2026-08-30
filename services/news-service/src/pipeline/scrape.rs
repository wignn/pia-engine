use std::sync::Arc;

use async_nats::jetstream::stream::RetentionPolicy;
use async_nats::jetstream::{self, stream};
use atlsd_eventbus::subjects;
use sqlx::PgPool;
use tracing::{info, warn};

use crate::config::Config;

pub fn is_supported(url: &str) -> bool {
    url.contains("fxstreet.com") || url.contains("investing.com")
}

pub async fn connect_publisher(url: &str) -> anyhow::Result<Arc<jetstream::Context>> {
    let client = async_nats::connect(url).await?;
    let jetstream = jetstream::new(client);
    jetstream
        .get_or_create_stream(stream::Config {
            name: "SCRAPE_JOBS".to_string(),
            subjects: vec![subjects::SCRAPE_JOBS.to_string()],
            retention: RetentionPolicy::WorkQueue,
            ..Default::default()
        })
        .await?;
    Ok(Arc::new(jetstream))
}

pub async fn publish_job(
    jetstream: &jetstream::Context,
    content_hash: &str,
    url: &str,
) -> anyhow::Result<()> {
    let job = atlsd_contracts::scrape::ScrapeJob {
        id: Some(content_hash.to_string()),
        url: url.to_string(),
    };
    let payload = serde_json::to_vec(&job)?;
    jetstream
        .publish(subjects::SCRAPE_JOBS.to_string(), payload.into())
        .await?
        .await?;
    Ok(())
}

/// The result payload is the shared atlsd-contracts type; only id + content
/// are consumed here.
type ScrapeResult = atlsd_contracts::scrape::ScrapeResult;

/// Subscribe to scrape.results and backfill original_content by content_hash.
/// Reconnect loop mirrors realtime.rs.
pub async fn run_consumer(cfg: Config, pool: PgPool) {
    use atlsd_eventbus::EventBusMode;
    if !matches!(
        EventBusMode::from_env_value(&cfg.eventbus_mode),
        EventBusMode::Nats | EventBusMode::Dual
    ) {
        return;
    }

    loop {
        match subscribe_loop(&cfg.nats_url, &pool).await {
            Ok(()) => {}
            Err(err) => warn!(error = %err, "scrape result consumer failed, reconnecting in 5s"),
        }
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}

async fn subscribe_loop(nats_url: &str, pool: &PgPool) -> anyhow::Result<()> {
    use futures_util::StreamExt;
    let client = async_nats::connect(nats_url).await?;
    let mut sub_legacy = client
        .subscribe(subjects::SCRAPE_RESULTS.to_string())
        .await?;
    let mut sub_enriched = client
        .subscribe(subjects::NEWS_ARTICLE_ENRICHED_V1.to_string())
        .await?;
    info!(
        legacy_subject = subjects::SCRAPE_RESULTS,
        enriched_subject = subjects::NEWS_ARTICLE_ENRICHED_V1,
        "news-service subscribed to scrape and enrichment results"
    );

    loop {
        tokio::select! {
            Some(message) = sub_legacy.next() => {
                let result: ScrapeResult = match serde_json::from_slice(&message.payload) {
                    Ok(result) => result,
                    Err(err) => {
                        warn!(error = %err, "failed to parse legacy scrape result");
                        continue;
                    }
                };
                if !result.ok {
                    warn!(id = %result.id, error = ?result.error, "scrape job reported failure");
                    continue;
                }
                let Some(content) = result
                    .news
                    .and_then(|n| n.content)
                    .filter(|c| !c.trim().is_empty())
                else {
                    continue;
                };
                let updated = sqlx::query(
                    "UPDATE news.forex_news_articles SET original_content = $1 WHERE content_hash = $2",
                )
                .bind(&content)
                .bind(&result.id)
                .execute(pool)
                .await?
                .rows_affected();
                if updated > 0 {
                    info!(content_hash = %result.id, "backfilled article content from legacy scrape");
                }
            }
            Some(message) = sub_enriched.next() => {
                let news: atlsd_contracts::macro_data::MacroNewsScraped = match serde_json::from_slice(&message.payload) {
                    Ok(item) => item,
                    Err(err) => {
                        warn!(error = %err, "failed to parse enriched news event");
                        continue;
                    }
                };
                let updated = sqlx::query(
                    r#"UPDATE news.forex_news_articles
                       SET original_content = COALESCE(NULLIF($1, ''), original_content),
                           media_url = COALESCE(NULLIF($2, ''), media_url)
                       WHERE content_hash = $3"#,
                )
                .bind(news.content.as_deref().unwrap_or(""))
                .bind(news.media_url.as_deref().unwrap_or(""))
                .bind(&news.id)
                .execute(pool)
                .await?
                .rows_affected();
                if updated > 0 {
                    info!(content_hash = %news.id, "enriched article content and media_url from JetStream");
                }
            }
            else => break,
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_scrapy_sources_are_supported() {
        assert!(is_supported("https://www.fxstreet.com/news/x"));
        assert!(is_supported("https://www.investing.com/news/y"));
        assert!(!is_supported("https://reuters.com/z"));
    }

    #[test]
    fn job_payload_serializes_via_shared_contract() {
        let job = atlsd_contracts::scrape::ScrapeJob {
            id: Some("hash\"1".to_string()),
            url: r#"https://x.com/a"b"#.to_string(),
        };
        let payload = serde_json::to_string(&job).unwrap();
        let value: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(value["id"], "hash\"1");
        assert_eq!(value["url"], r#"https://x.com/a"b"#);
        let parsed: atlsd_contracts::scrape::ScrapeJob = serde_json::from_str(&payload).unwrap();
        assert_eq!(parsed, job);
    }
}
