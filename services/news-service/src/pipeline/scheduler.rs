use sqlx::PgPool;
use tokio::time::{self, Duration};
use tracing::{error, info, warn};

use crate::config::Config;

use super::analysis::AnalyzerClient;
use super::persistence;
use super::rss::RssClient;
use super::sources;

pub async fn run(cfg: Config, pool: PgPool) {
    let rss_client = match RssClient::new() {
        Ok(client) => client,
        Err(err) => {
            error!(error = %err, "failed to create RSS client");
            return;
        }
    };
    let analyzer = AnalyzerClient::new(rss_client.http_client(), cfg.ai_service_url.clone());

    info!(
        rss_interval_sec = cfg.rss_fetch_sec,
        stock_interval_sec = cfg.stock_fetch_sec,
        analyzer_enabled = cfg.ai_service_url.is_some(),
        "news ingestion pipeline running"
    );

    run_once(&pool, &rss_client, &analyzer).await;

    let mut interval = time::interval(Duration::from_secs(cfg.rss_fetch_sec));
    loop {
        interval.tick().await;
        run_once(&pool, &rss_client, &analyzer).await;
    }
}

async fn run_once(pool: &PgPool, rss_client: &RssClient, analyzer: &AnalyzerClient) {
    let sources = match sources::due_sources(pool).await {
        Ok(sources) => sources,
        Err(err) => {
            error!(error = %err, "failed to load news sources");
            return;
        }
    };

    for source in sources {
        match poll_source(pool, rss_client, analyzer, &source).await {
            Ok(inserted) => info!(source = %source.name, inserted, "news source polled"),
            Err(err) => warn!(source = %source.name, error = %err, "news source poll failed"),
        }
    }
}

async fn poll_source(
    pool: &PgPool,
    rss_client: &RssClient,
    analyzer: &AnalyzerClient,
    source: &sources::NewsSource,
) -> anyhow::Result<usize> {
    let result = rss_client.fetch(source).await?;
    if !(200..300).contains(&result.status) {
        sources::record_error(
            pool,
            source,
            result.status,
            result.latency_ms,
            "non-success RSS response",
        )
        .await?;
        return Ok(0);
    }

    let mut inserted = 0usize;
    for article in result.articles {
        let analysis = analyzer.analyze(&article).await;
        inserted += persistence::insert_forex_article(pool, source, &article, &analysis).await?;
    }

    sources::record_success(pool, source, result.status, result.latency_ms).await?;
    Ok(inserted)
}
