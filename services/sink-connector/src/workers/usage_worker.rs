use crate::{config::Config, dlq};
use async_nats::jetstream::{self, consumer::PullConsumer, AckKind};
use atlsd_contracts::platform::ApiUsageRequestedEvent;
use futures_util::StreamExt;
use sqlx::{PgPool, QueryBuilder};
use std::time::Duration;
use tracing::{error, info, warn};

pub async fn run(pool: PgPool, js: jetstream::Context, config: Config) -> anyhow::Result<()> {
    let stream = js
        .get_stream(atlsd_eventbus::subjects::ATLSD_PLATFORM_STREAM)
        .await?;

    let consumer: PullConsumer = stream
        .get_or_create_consumer(
            &config.usage_consumer_name,
            async_nats::jetstream::consumer::pull::Config {
                durable_name: Some(config.usage_consumer_name.clone()),
                filter_subject: atlsd_eventbus::subjects::USAGE_API_REQUESTED_V1.to_string(),
                ack_wait: Duration::from_secs(config.usage_ack_wait_sec),
                max_deliver: config.usage_max_deliver,
                max_ack_pending: 1000,
                ..Default::default()
            },
        )
        .await?;

    info!(
        stream = atlsd_eventbus::subjects::ATLSD_PLATFORM_STREAM,
        consumer = %config.usage_consumer_name,
        "usage batch worker ready"
    );

    loop {
        let mut messages = match consumer
            .fetch()
            .max_messages(config.usage_batch_size)
            .expires(Duration::from_millis(config.usage_batch_timeout_ms))
            .messages()
            .await
        {
            Ok(msgs) => msgs,
            Err(err) => {
                warn!(error = %err, "usage consumer fetch error");
                tokio::time::sleep(Duration::from_millis(500)).await;
                continue;
            }
        };

        let mut batch: Vec<(ApiUsageRequestedEvent, jetstream::Message)> = Vec::new();

        while let Some(result) = messages.next().await {
            let message = match result {
                Ok(msg) => msg,
                Err(err) => {
                    warn!(error = %err, "error retrieving usage message from stream");
                    continue;
                }
            };

            match serde_json::from_slice::<ApiUsageRequestedEvent>(&message.payload) {
                Ok(event) => {
                    batch.push((event, message));
                }
                Err(decode_err) => {
                    let err_str = decode_err.to_string();
                    warn!(error = %err_str, "poison usage event (decode), moving to DLQ");
                    let _ = dlq::publish_dlq(
                        &js,
                        atlsd_eventbus::subjects::USAGE_DLQ_V1,
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

        if let Err(err) = flush_usage_batch(&pool, &batch).await {
            error!(error = %err, count = batch.len(), "failed to flush usage batch to postgres, nacking batch");
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

async fn flush_usage_batch(
    pool: &PgPool,
    batch: &[(ApiUsageRequestedEvent, jetstream::Message)],
) -> anyhow::Result<()> {
    if batch.is_empty() {
        return Ok(());
    }

    let mut query_builder: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
        "INSERT INTO auth.usage_logs (user_id, api_key_id, endpoint, method, status_code, response_ms, created_at) ",
    );

    query_builder.push_values(batch.iter(), |mut b, (evt, _msg)| {
        b.push_bind(evt.user_id)
            .push_bind(evt.api_key_id)
            .push_bind(&evt.endpoint)
            .push_bind(&evt.method)
            .push_bind(evt.status_code)
            .push_bind(evt.response_ms)
            .push_bind(evt.requested_at);
    });

    query_builder.build().execute(pool).await?;
    Ok(())
}
