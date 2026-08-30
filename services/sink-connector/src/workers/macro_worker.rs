use crate::{config::Config, dlq, writer};
use async_nats::jetstream::{self, consumer::PullConsumer, AckKind};
use atlsd_contracts::macro_data::MacroEvent;
use futures_util::StreamExt;
use sqlx::PgPool;
use std::time::Duration;
use tracing::{error, info, warn};

pub async fn run(pool: PgPool, js: jetstream::Context, config: Config) -> anyhow::Result<()> {
    let stream = js
        .get_stream(atlsd_eventbus::subjects::ATLSD_MACRO_STREAM)
        .await?;

    let consumer: PullConsumer = stream
        .get_or_create_consumer(
            &config.macro_consumer_name,
            async_nats::jetstream::consumer::pull::Config {
                durable_name: Some(config.macro_consumer_name.clone()),
                filter_subject: atlsd_eventbus::subjects::MACRO_EVENTS_V1.to_string(),
                ack_wait: Duration::from_secs(config.macro_ack_wait_sec),
                max_deliver: config.macro_max_deliver,
                max_ack_pending: 200,
                ..Default::default()
            },
        )
        .await?;

    info!(
        stream = atlsd_eventbus::subjects::ATLSD_MACRO_STREAM,
        consumer = %config.macro_consumer_name,
        "macro batch worker ready"
    );

    loop {
        let mut messages = match consumer
            .fetch()
            .max_messages(config.macro_batch_size)
            .expires(Duration::from_secs(2))
            .messages()
            .await
        {
            Ok(msgs) => msgs,
            Err(err) => {
                warn!(error = %err, "macro consumer fetch error");
                tokio::time::sleep(Duration::from_millis(500)).await;
                continue;
            }
        };

        while let Some(result) = messages.next().await {
            let message = match result {
                Ok(msg) => msg,
                Err(err) => {
                    warn!(error = %err, "error retrieving macro message from stream");
                    continue;
                }
            };

            match handle_single_event(&pool, &js, &message).await {
                Ok(()) => {
                    let _ = message.ack().await;
                }
                Err(WorkerError::Validation(err)) => {
                    warn!(error = %err, subject = %message.subject, "poison macro event (validation), moving to DLQ");
                    let _ = dlq::publish_dlq(
                        &js,
                        atlsd_eventbus::subjects::MACRO_DLQ_V1,
                        message.subject.as_str(),
                        &err,
                        message.payload.as_ref(),
                    )
                    .await;
                    let _ = message.ack().await;
                }
                Err(WorkerError::Decode(err)) => {
                    let error_text = err.to_string();
                    warn!(error = %error_text, subject = %message.subject, "poison macro event (decode), moving to DLQ");
                    let _ = dlq::publish_dlq(
                        &js,
                        atlsd_eventbus::subjects::MACRO_DLQ_V1,
                        message.subject.as_str(),
                        &error_text,
                        message.payload.as_ref(),
                    )
                    .await;
                    let _ = message.ack().await;
                }
                Err(err) => {
                    error!(error = %err, subject = %message.subject, "transient macro DB write failure, nacking");
                    let _ = message.ack_with(AckKind::Nak(Some(Duration::from_secs(5)))).await;
                }
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum WorkerError {
    #[error("validation error: {0}")]
    Validation(String),

    #[error("json decode error: {0}")]
    Decode(#[from] serde_json::Error),

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("other error: {0}")]
    Other(#[from] anyhow::Error),
}

async fn handle_single_event(
    pool: &PgPool,
    js: &jetstream::Context,
    message: &jetstream::Message,
) -> Result<(), WorkerError> {
    let event: MacroEvent = serde_json::from_slice(&message.payload)?;
    event.validate().map_err(WorkerError::Validation)?;
    writer::write_event(pool, js, &event)
        .await
        .map_err(WorkerError::Other)?;
    Ok(())
}
