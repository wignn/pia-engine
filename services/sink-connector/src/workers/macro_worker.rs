use crate::{config::Config, dlq};
use async_nats::jetstream::{self, consumer::PullConsumer, AckKind};
use atlsd_contracts::macro_data::{MacroEvent, MacroPayload};
use atlsd_eventbus::subjects;
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
                    let _ = message
                        .ack_with(AckKind::Nak(Some(Duration::from_secs(5))))
                        .await;
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
    write_macro_event(pool, js, &event)
        .await
        .map_err(WorkerError::Other)?;
    Ok(())
}

pub async fn write_macro_event(
    pool: &PgPool,
    js: &jetstream::Context,
    event: &MacroEvent,
) -> anyhow::Result<()> {
    match event.decode_payload()? {
        MacroPayload::Rate(rate) => {
            sqlx::query(
                r#"INSERT INTO macro.macro_rates
                   (source, country, tenor, date, value, unit, raw_series_id, created_at, updated_at)
                   VALUES ($1,$2,$3,$4,$5,$6,$7,NOW(),NOW())
                   ON CONFLICT (source,country,tenor,date) DO UPDATE SET
                     value=EXCLUDED.value, unit=EXCLUDED.unit, raw_series_id=EXCLUDED.raw_series_id, updated_at=NOW()"#,
            )
            .bind(&event.source)
            .bind(&rate.country)
            .bind(&rate.tenor)
            .bind(rate.date)
            .bind(rate.value)
            .bind(&rate.unit)
            .bind(&rate.series_id)
            .execute(pool)
            .await?;
        }
        MacroPayload::Spread(spread) => {
            sqlx::query(
                r#"INSERT INTO macro.macro_rate_spreads
                   (country, spread, date, value, created_at, updated_at)
                   VALUES ($1,$2,$3,$4,NOW(),NOW())
                   ON CONFLICT (country,spread,date) DO UPDATE SET value=EXCLUDED.value, updated_at=NOW()"#,
            )
            .bind(&spread.country)
            .bind(&spread.spread)
            .bind(spread.date)
            .bind(spread.value)
            .execute(pool)
            .await?;
        }
        MacroPayload::Series(series) => {
            let mut tx = pool.begin().await?;
            sqlx::query(
                r#"INSERT INTO macro.macro_series
                   (id, provider, title, category, units, frequency, last_synced_at, created_at, updated_at)
                   VALUES ($1,$2,$3,$4,$5,$6,$7,NOW(),NOW())
                   ON CONFLICT (id) DO UPDATE SET title=EXCLUDED.title, category=EXCLUDED.category,
                     units=EXCLUDED.units, frequency=EXCLUDED.frequency, last_synced_at=EXCLUDED.last_synced_at,
                     updated_at=NOW()"#,
            )
            .bind(&series.series_id)
            .bind(&event.source)
            .bind(&series.title)
            .bind(&series.category)
            .bind(&series.units)
            .bind(&series.frequency)
            .bind(event.observed_at)
            .execute(&mut *tx)
            .await?;
            sqlx::query(
                r#"INSERT INTO macro.macro_observations
                   (series_id, observation_date, value, raw_value, created_at, updated_at)
                   VALUES ($1,$2,$3,$4,NOW(),NOW())
                   ON CONFLICT (series_id,observation_date) DO UPDATE SET value=EXCLUDED.value,
                     raw_value=EXCLUDED.raw_value, updated_at=NOW()"#,
            )
            .bind(&series.series_id)
            .bind(series.date)
            .bind(series.value)
            .bind(&series.raw_value)
            .execute(&mut *tx)
            .await?;
            tx.commit().await?;
        }
        MacroPayload::Bond(bond) => {
            let id = format!("{}_{}", bond.country.to_lowercase(), bond.as_of);
            sqlx::query(
                r#"INSERT INTO macro.macro_bonds
                   (id, country, as_of, raw_json, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, NOW(), NOW())
                   ON CONFLICT (id) DO UPDATE SET
                     raw_json = EXCLUDED.raw_json, updated_at = NOW()"#,
            )
            .bind(&id)
            .bind(&bond.country)
            .bind(bond.as_of)
            .bind(&bond.raw)
            .execute(pool)
            .await?;
        }
        MacroPayload::NewsScraped(news) => {
            let payload = serde_json::to_vec(&news)?;
            js.publish(
                subjects::NEWS_ARTICLE_ENRICHED_V1.to_string(),
                payload.into(),
            )
            .await?
            .await?;
            info!(
                article_id = %news.id,
                subject = subjects::NEWS_ARTICLE_ENRICHED_V1,
                "dispatched news enrichment event to news-service"
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlsd_contracts::macro_data::MacroEvent;
    use chrono::{NaiveDate, Utc};

    #[test]
    fn decodes_macro_rate_payload_correctly() {
        let event = MacroEvent {
            event_id: "rate-test-1".to_string(),
            schema_version: 1,
            source: "fred".to_string(),
            feed_type: "rate".to_string(),
            observed_at: Utc::now(),
            published_at: Utc::now(),
            payload: serde_json::json!({
                "country": "US",
                "tenor": "10Y",
                "date": "2026-08-25",
                "value": 4.25,
                "unit": "percent",
                "series_id": "DGS10"
            }),
        };

        let decoded = event.decode_payload().expect("should decode rate payload");
        match decoded {
            MacroPayload::Rate(rate) => {
                assert_eq!(rate.country, "US");
                assert_eq!(rate.tenor, "10Y");
                assert_eq!(rate.value, 4.25);
                assert_eq!(rate.date, NaiveDate::from_ymd_opt(2026, 8, 25).unwrap());
            }
            _ => panic!("expected MacroPayload::Rate"),
        }
    }
}
