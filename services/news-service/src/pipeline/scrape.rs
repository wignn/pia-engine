use std::sync::Arc;

use async_nats::jetstream::stream::RetentionPolicy;
use async_nats::jetstream::{self, stream};
use atlsd_eventbus::subjects;
use serde::Deserialize;
use sqlx::PgPool;
use tracing::{info, warn};

use crate::config::Config;

/// scrapy only knows these sources (see services/scrapy/src/scraper/mod.rs).
/// No point publishing a job it will reject.
pub fn is_supported(url: &str) -> bool {
    url.contains("fxstreet.com") || url.contains("investing.com")
}

/// Publish a fetch job keyed by the article's content_hash so the result loop
/// can match the extracted content back to the row. Best-effort: a NATS hiccup
/// must not fail news ingestion.
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
    let payload = format!(
        r#"{{"id":{},"url":{}}}"#,
        json_str(content_hash),
        json_str(url)
    );
    jetstream
        .publish(
            subjects::SCRAPE_JOBS.to_string(),
            payload.into_bytes().into(),
        )
        .await?
        .await?;
    Ok(())
}

/// scrapy's ScrapeResult (services/scrapy/src/models/mod.rs). We only need id + content.
#[derive(Deserialize)]
struct ScrapeResult {
    id: String,
    ok: bool,
    news: Option<News>,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Deserialize)]
struct News {
    content: Option<String>,
}

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
    let mut sub = client
        .subscribe(subjects::SCRAPE_RESULTS.to_string())
        .await?;
    info!(
        subject = subjects::SCRAPE_RESULTS,
        "news-service subscribed to scrape results"
    );

    while let Some(message) = sub.next().await {
        let result: ScrapeResult = match serde_json::from_slice(&message.payload) {
            Ok(result) => result,
            Err(err) => {
                warn!(error = %err, "failed to parse scrape result");
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
            info!(content_hash = %result.id, "backfilled article content from scrapy");
        }
    }
    Ok(())
}

/// Minimal JSON string escaping (quotes + backslashes) — the only two
/// characters that can break a URL or hex hash embedded in JSON.
fn json_str(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
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
    fn job_payload_is_valid_json_and_escapes_quotes() {
        let payload = format!(
            r#"{{"id":{},"url":{}}}"#,
            json_str("hash\"1"),
            json_str(r#"https://x.com/a"b"#)
        );
        let value: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(value["id"], "hash\"1");
        assert_eq!(value["url"], r#"https://x.com/a"b"#);
    }
}
