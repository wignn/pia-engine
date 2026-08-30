use crate::{config::Config, dlq};
use async_nats::jetstream::{self, consumer::PullConsumer, AckKind};
use atlsd_contracts::macro_data::MacroNewsScraped;
use futures_util::StreamExt;
use sqlx::PgPool;
use std::time::Duration;
use tracing::{error, info, warn};

pub async fn run(pool: PgPool, js: jetstream::Context, config: Config) -> anyhow::Result<()> {
    let stream = js
        .get_stream(atlsd_eventbus::subjects::ATLSD_NEWS_STREAM)
        .await?;

    let consumer: PullConsumer = stream
        .get_or_create_consumer(
            &config.news_consumer_name,
            async_nats::jetstream::consumer::pull::Config {
                durable_name: Some(config.news_consumer_name.clone()),
                filter_subject: atlsd_eventbus::subjects::NEWS_ARTICLE_ENRICHED_V1.to_string(),
                ack_wait: Duration::from_secs(config.news_ack_wait_sec),
                max_deliver: config.news_max_deliver,
                max_ack_pending: 500,
                ..Default::default()
            },
        )
        .await?;

    info!(
        stream = atlsd_eventbus::subjects::ATLSD_NEWS_STREAM,
        consumer = %config.news_consumer_name,
        "news enrichment batch worker ready"
    );

    loop {
        let mut messages = match consumer
            .fetch()
            .max_messages(config.news_batch_size)
            .expires(Duration::from_millis(config.news_batch_timeout_ms))
            .messages()
            .await
        {
            Ok(msgs) => msgs,
            Err(err) => {
                warn!(error = %err, "news consumer fetch error");
                tokio::time::sleep(Duration::from_millis(500)).await;
                continue;
            }
        };

        let mut batch: Vec<(MacroNewsScraped, jetstream::Message)> = Vec::new();

        while let Some(result) = messages.next().await {
            let message = match result {
                Ok(msg) => msg,
                Err(err) => {
                    warn!(error = %err, "error retrieving news message from stream");
                    continue;
                }
            };

            match serde_json::from_slice::<MacroNewsScraped>(&message.payload) {
                Ok(event) => {
                    batch.push((event, message));
                }
                Err(decode_err) => {
                    let err_str = decode_err.to_string();
                    warn!(error = %err_str, "poison news event (decode), moving to DLQ");
                    let _ = dlq::publish_dlq(
                        &js,
                        atlsd_eventbus::subjects::NEWS_DLQ_V1,
                        message.subject.as_str(),
                        &err_str,
                        message.payload.as_ref(),
                    )
                    .await;
                    let _ = message.ack().await;
                }
            }
        }

        if batch.is_empty() {
            continue;
        }

        if let Err(err) = flush_news_batch(&pool, &batch).await {
            error!(error = %err, count = batch.len(), "failed to flush news batch to postgres, nacking batch");
            for (_, msg) in batch {
                let _ = msg.ack_with(AckKind::Nak(Some(Duration::from_secs(3)))).await;
            }
        } else {
            for (_, msg) in batch {
                let _ = msg.ack().await;
            }
        }
    }
}

async fn flush_news_batch(
    pool: &PgPool,
    batch: &[(MacroNewsScraped, jetstream::Message)],
) -> anyhow::Result<()> {
    if batch.is_empty() {
        return Ok(());
    }

    let mut tx = pool.begin().await?;

    for (item, _) in batch {
        let content = item.content.as_deref().filter(|c| !c.trim().is_empty());
        let media = item.media_url.as_deref().filter(|m| !m.trim().is_empty());

        if content.is_some() || media.is_some() {
            sqlx::query(
                r#"UPDATE news.forex_news_articles
                   SET original_content = COALESCE($1, original_content),
                       media_url = COALESCE($2, media_url)
                   WHERE content_hash = $3"#,
            )
            .bind(content)
            .bind(media)
            .bind(&item.id)
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;
    Ok(())
}
