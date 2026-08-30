use std::sync::Arc;

use async_nats::jetstream::stream::RetentionPolicy;
use async_nats::jetstream::{self, stream};
use atlsd_eventbus::subjects;

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
