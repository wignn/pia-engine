use chrono::Utc;
use redis::AsyncCommands;
use sqlx::{PgPool, Postgres, QueryBuilder};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::tenant::context::TenantContext;

#[derive(Debug, Clone)]
pub struct UsageEvent {
    pub user_id: Uuid,
    pub api_key_id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub status_code: i32,
    pub response_ms: i32,
}

#[derive(Clone)]
pub struct UsageTracker {
    tx: mpsc::Sender<UsageEvent>,
    redis_client: Option<redis::Client>,
}

impl UsageTracker {
    pub fn new(db: PgPool, redis_client: Option<redis::Client>) -> Self {
        let (tx, mut rx) = mpsc::channel::<UsageEvent>(8_192);

        tokio::spawn(async move {
            const BATCH_SIZE: usize = 200;
            const FLUSH_MS: u64 = 750;

            let mut batch = Vec::with_capacity(BATCH_SIZE);
            let mut ticker = tokio::time::interval(std::time::Duration::from_millis(FLUSH_MS));

            loop {
                tokio::select! {
                    maybe_evt = rx.recv() => {
                        match maybe_evt {
                            Some(evt) => {
                                batch.push(evt);
                                if batch.len() >= BATCH_SIZE {
                                    flush_batch(&db, &mut batch).await;
                                }
                            }
                            None => {
                                if !batch.is_empty() {
                                    flush_batch(&db, &mut batch).await;
                                }
                                break;
                            }
                        }
                    }
                    _ = ticker.tick() => {
                        if !batch.is_empty() {
                            flush_batch(&db, &mut batch).await;
                        }
                    }
                }
            }
        });

        Self { tx, redis_client }
    }

    pub async fn enqueue(&self, event: UsageEvent) {
        if let Err(err) = self.tx.send(event).await {
            tracing::warn!(error = %err, "usage event dropped: queue closed");
        }
    }

    pub async fn try_consume_daily_quota(&self, tenant: &TenantContext) -> bool {
        if tenant.is_admin {
            return true;
        }

        let Some(redis_client) = &self.redis_client else {
            return true;
        };

        let key = format!(
            "usage:daily:{}:{}",
            tenant.user_id,
            Utc::now().format("%Y-%m-%d")
        );
        let ttl = seconds_until_next_utc_day();

        let mut conn = match redis_client.get_multiplexed_tokio_connection().await {
            Ok(conn) => conn,
            Err(err) => {
                tracing::warn!(error = %err, "quota check redis connect failed; fail-open");
                return true;
            }
        };

        let count: i64 = match redis::Script::new(
            "local v=redis.call('INCR', KEYS[1]); if v==1 then redis.call('EXPIRE', KEYS[1], ARGV[1]); end; return v"
        )
        .key(&key)
        .arg(ttl)
        .invoke_async(&mut conn)
        .await
        {
            Ok(v) => v,
            Err(err) => {
                tracing::warn!(error = %err, "quota check redis script failed; fail-open");
                return true;
            }
        };

        count <= i64::from(tenant.requests_per_day)
    }

    pub async fn daily_count(&self, user_id: Uuid) -> Option<i64> {
        let redis_client = self.redis_client.as_ref()?;
        let key = format!(
            "usage:daily:{}:{}",
            user_id,
            Utc::now().format("%Y-%m-%d")
        );

        let mut conn = redis_client.get_multiplexed_tokio_connection().await.ok()?;
        conn.get::<_, Option<i64>>(key).await.ok().flatten()
    }
}

async fn flush_batch(db: &PgPool, batch: &mut Vec<UsageEvent>) {
    let mut builder: QueryBuilder<'_, Postgres> = QueryBuilder::new(
        "INSERT INTO usage_logs (user_id, api_key_id, endpoint, method, status_code, response_ms) ",
    );

    builder.push_values(batch.iter(), |mut b, evt| {
        b.push_bind(evt.user_id)
            .push_bind(Some(evt.api_key_id))
            .push_bind(&evt.endpoint)
            .push_bind(&evt.method)
            .push_bind(evt.status_code)
            .push_bind(Some(evt.response_ms));
    });

    if let Err(err) = builder.build().execute(db).await {
        tracing::warn!(error = %err, size = batch.len(), "usage batch flush failed");
    }

    batch.clear();
}

fn seconds_until_next_utc_day() -> i64 {
    let now = Utc::now();
    let next = (now.date_naive() + chrono::Duration::days(1))
        .and_hms_opt(0, 0, 0)
        .expect("valid midnight");
    (next - now.naive_utc()).num_seconds().max(1)
}
