use atlsd_eventbus::nats::dedup_headers;
use atlsd_eventbus::EventBusMode;
use sqlx::PgPool;
use tracing::{error, info, warn};

use crate::config::Config;

const RELAY_BATCH: i64 = 100;

struct OutboxRow {
    id: i64,
    aggregate_type: String,
    aggregate_id: String,
    subject: String,
    payload: String,
}

pub async fn run(
    cfg: Config,
    pool: PgPool,
    metrics: std::sync::Arc<atlsd_observability::MetricsRegistry>,
) {
    if !matches!(
        EventBusMode::from_env_value(&cfg.eventbus_mode),
        EventBusMode::Nats | EventBusMode::Dual
    ) {
        return;
    }

    loop {
        match async_nats::connect(&cfg.nats_url).await {
            Ok(client) => {
                info!(url = %cfg.nats_url, "news-service outbox relay connected to NATS");
                relay_loop(&cfg, &pool, client, &metrics).await;
            }
            Err(err) => {
                error!(error = %err, url = %cfg.nats_url, "news-service NATS connection failed");
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
    }
}

async fn relay_loop(
    cfg: &Config,
    pool: &PgPool,
    client: async_nats::Client,
    metrics: &atlsd_observability::MetricsRegistry,
) {
    let jetstream = async_nats::jetstream::new(client);
    let mut interval =
        tokio::time::interval(std::time::Duration::from_secs(cfg.realtime_poll_sec.max(1)));

    loop {
        interval.tick().await;
        let rows = match fetch_unpublished(pool, RELAY_BATCH).await {
            Ok(rows) => rows,
            Err(err) => {
                warn!(error = %err, "outbox relay fetch failed");
                continue;
            }
        };
        metrics.set_gauge(
            "atlsd_news_outbox_backlog",
            "Unpublished rows currently held in the outbox.",
            rows.len() as u64,
        );
        if rows.is_empty() {
            continue;
        }

        info!(count = rows.len(), "outbox relay publishing batch");
        for row in rows {
            let msg_id = outbox_msg_id(&row.aggregate_type, &row.aggregate_id);
            let publish = jetstream
                .publish_with_headers(
                    row.subject.clone(),
                    dedup_headers(&msg_id),
                    row.payload.clone().into(),
                )
                .await;

            match publish {
                Ok(ack_future) => match ack_future.await {
                    Ok(_ack) => {
                        metrics.inc(
                            "atlsd_news_outbox_published_total",
                            "Outbox rows published to JetStream.",
                        );
                        if let Err(err) = mark_published(pool, row.id).await {
                            warn!(error = %err, outbox_id = row.id, "outbox mark-published failed; row will be re-published");
                        }
                    }
                    Err(err) => {
                        metrics.inc(
                            "atlsd_news_outbox_publish_failures_total",
                            "Outbox publishes that were not acknowledged.",
                        );
                        warn!(error = %err, outbox_id = row.id, subject = %row.subject, "outbox publish not acknowledged; will retry");
                    }
                },
                Err(err) => {
                    metrics.inc(
                        "atlsd_news_outbox_publish_failures_total",
                        "Outbox publishes that were not acknowledged.",
                    );
                    warn!(error = %err, outbox_id = row.id, subject = %row.subject, "outbox publish failed; will retry");
                }
            }
        }
    }
}

fn outbox_msg_id(aggregate_type: &str, aggregate_id: &str) -> String {
    format!("{aggregate_type}:{aggregate_id}")
}

async fn fetch_unpublished(pool: &PgPool, limit: i64) -> Result<Vec<OutboxRow>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (i64, String, String, String, String)>(
        "SELECT id, aggregate_type, aggregate_id, subject, payload::text \
         FROM platform.outbox_events \
         WHERE published_at IS NULL \
         ORDER BY id \
         LIMIT $1",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| OutboxRow {
            id: row.0,
            aggregate_type: row.1,
            aggregate_id: row.2,
            subject: row.3,
            payload: row.4,
        })
        .collect())
}

async fn mark_published(pool: &PgPool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE platform.outbox_events SET published_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outbox_msg_id_combines_aggregate_identity() {
        assert_eq!(
            outbox_msg_id("news.forex_article", "42"),
            "news.forex_article:42"
        );
    }
}
