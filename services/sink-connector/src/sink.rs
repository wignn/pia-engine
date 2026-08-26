use crate::{config::Config, dlq, writer};
use async_nats::jetstream::{self, consumer::PullConsumer, AckKind};
use atlsd_contracts::macro_data::MacroEvent;
use futures_util::StreamExt;
use std::time::Duration;
use tracing::{info, warn};

#[derive(Debug, thiserror::Error)]
pub enum SinkError {
    #[error("validation error: {0}")]
    Validation(String),

    #[error("json decode error: {0}")]
    Decode(#[from] serde_json::Error),

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("nats error: {0}")]
    Nats(String),

    #[error("other error: {0}")]
    Other(#[from] anyhow::Error),
}

pub async fn run(config: Config) -> anyhow::Result<()> {
    let client = async_nats::connect(&config.nats_url).await?;
    atlsd_eventbus::nats::init_jetstream_streams(&client).await?;
    let js = jetstream::new(client.clone());
    let stream = js
        .get_stream(atlsd_eventbus::subjects::ATLSD_MACRO_STREAM)
        .await?;
    let consumer: PullConsumer = stream
        .get_or_create_consumer(
            &config.consumer_name,
            async_nats::jetstream::consumer::pull::Config {
                durable_name: Some(config.consumer_name.clone()),
                filter_subject: atlsd_eventbus::subjects::MACRO_EVENTS_V1.to_string(),
                ack_wait: Duration::from_secs(config.ack_wait_sec),
                max_deliver: config.max_deliver,
                max_ack_pending: 100,
                ..Default::default()
            },
        )
        .await?;
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;

    info!(
        stream = atlsd_eventbus::subjects::ATLSD_MACRO_STREAM,
        "macro sink consumer ready"
    );
    loop {
        let mut messages = consumer
            .fetch()
            .max_messages(1)
            .expires(Duration::from_secs(2))
            .messages()
            .await?;
        let Some(result) = messages.next().await else {
            continue;
        };
        let message = result.map_err(|e| anyhow::anyhow!(e))?;
        match handle_message(&pool, &message).await {
            Ok(()) => message
                .ack()
                .await
                .map_err(|e| SinkError::Nats(e.to_string()))?,
            Err(SinkError::Validation(err)) => {
                warn!(error = %err, subject = %message.subject, "poison macro event (validation), moving to DLQ");
                if let Err(dlq_err) = dlq::publish(
                    &js,
                    dlq::DeadLetter {
                        source_subject: message.subject.as_str(),
                        error: &err,
                        payload: message.payload.as_ref(),
                    },
                )
                .await
                {
                    warn!(error = %dlq_err, "failed to publish macro DLQ event");
                }
                message
                    .ack()
                    .await
                    .map_err(|e| SinkError::Nats(e.to_string()))?;
            }
            Err(SinkError::Decode(err)) => {
                warn!(error = %err, subject = %message.subject, "poison macro event (decode), moving to DLQ");
                let error_text = err.to_string();
                if let Err(dlq_err) = dlq::publish(
                    &js,
                    dlq::DeadLetter {
                        source_subject: message.subject.as_str(),
                        error: &error_text,
                        payload: message.payload.as_ref(),
                    },
                )
                .await
                {
                    warn!(error = %dlq_err, "failed to publish macro DLQ event");
                }
                message
                    .ack()
                    .await
                    .map_err(|e| SinkError::Nats(e.to_string()))?;
            }
            Err(err) => {
                warn!(error = %err, subject = %message.subject, "transient sink error, nabbing for retry");
                message
                    .ack_with(AckKind::Nak(Some(Duration::from_secs(5))))
                    .await
                    .map_err(|e| SinkError::Nats(e.to_string()))?;
            }
        }
    }
}

async fn handle_message(
    pool: &sqlx::PgPool,
    message: &jetstream::Message,
) -> Result<(), SinkError> {
    let event: MacroEvent = serde_json::from_slice(&message.payload)?;
    event.validate().map_err(SinkError::Validation)?;
    writer::write_event(pool, &event).await?;
    Ok(())
}
