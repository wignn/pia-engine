use async_nats::jetstream::{self, Context as JetStreamContext};
use atlsd_contracts::platform::ApiUsageRequestedEvent;
use chrono::Utc;
use tokio::sync::mpsc;
use uuid::Uuid;

use atlsd_domain::tenant::TenantContext;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateLimitDecision {
    Allowed {
        min_limit: u32,
        min_remaining: u32,
        day_limit: u32,
        day_remaining: u32,
        reset_seconds: u64,
    },
    MinuteExceeded {
        limit: u32,
        reset_seconds: u64,
    },
    DailyQuotaExceeded {
        limit: u32,
    },
    Bypassed,
}

impl RateLimitDecision {
    #[allow(dead_code)]
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allowed { .. } | Self::Bypassed)
    }
}

impl UsageTracker {
    pub fn new(js: Option<JetStreamContext>, redis_client: Option<redis::Client>) -> Self {
        let (tx, mut rx) = mpsc::channel::<UsageEvent>(8_192);
        tokio::spawn(async move {
            let mut batch = Vec::with_capacity(200);
            let mut ticker = tokio::time::interval(std::time::Duration::from_millis(500));
            loop {
                tokio::select! {
                    maybe_evt = rx.recv() => match maybe_evt {
                        Some(evt) => {
                            batch.push(evt);
                            if batch.len() >= 200 {
                                if let Some(context) = &js {
                                    flush_to_jetstream(context, &mut batch).await;
                                } else {
                                    batch.clear();
                                }
                            }
                        }
                        None => {
                            if !batch.is_empty() {
                                if let Some(context) = &js {
                                    flush_to_jetstream(context, &mut batch).await;
                                } else {
                                    batch.clear();
                                }
                            }
                            break;
                        }
                    },
                    _ = ticker.tick() => {
                        if !batch.is_empty() {
                            if let Some(context) = &js {
                                flush_to_jetstream(context, &mut batch).await;
                            } else {
                                batch.clear();
                            }
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

    pub async fn check_rate_limit(&self, tenant: &TenantContext) -> RateLimitDecision {
        if tenant.is_admin {
            return RateLimitDecision::Bypassed;
        }
        let Some(redis_client) = &self.redis_client else {
            return RateLimitDecision::Bypassed;
        };
        let minute_key = format!(
            "rate:min:{}:{}",
            tenant.user_id,
            Utc::now().format("%Y%m%d%H%M")
        );
        let daily_key = format!(
            "usage:daily:{}:{}",
            tenant.user_id,
            Utc::now().format("%Y-%m-%d")
        );
        let daily_ttl = seconds_until_next_utc_day();
        let mut conn = match redis_client.get_multiplexed_tokio_connection().await {
            Ok(conn) => conn,
            Err(err) => {
                tracing::warn!(error = %err, "quota redis connect failed; fail-open");
                return RateLimitDecision::Bypassed;
            }
        };

        // Dual-tier rate limiter (per-minute limit + daily quota)
        let script = r#"
            local min_val = redis.call('INCR', KEYS[1])
            if min_val == 1 then
                redis.call('EXPIRE', KEYS[1], 60)
            end
            local min_limit = tonumber(ARGV[1])
            local min_ttl = redis.call('TTL', KEYS[1])
            if min_val > min_limit then
                return {-1, min_limit, min_ttl}
            end

            local day_val = redis.call('INCR', KEYS[2])
            if day_val == 1 then
                redis.call('EXPIRE', KEYS[2], tonumber(ARGV[3]))
            end
            local day_limit = tonumber(ARGV[2])
            if day_val > day_limit then
                return {-2, day_limit, 0}
            end

            local min_rem = math.max(0, min_limit - min_val)
            local day_rem = math.max(0, day_limit - day_val)
            return {0, min_rem, day_rem, min_ttl}
        "#;

        let res: Result<Vec<i64>, _> = redis::Script::new(script)
            .key(&minute_key)
            .key(&daily_key)
            .arg(i64::from(tenant.rate_limit_per_min))
            .arg(i64::from(tenant.requests_per_day))
            .arg(daily_ttl)
            .invoke_async(&mut conn)
            .await;

        match res {
            Ok(vals) if !vals.is_empty() => match vals[0] {
                0 => RateLimitDecision::Allowed {
                    min_limit: tenant.rate_limit_per_min.max(0) as u32,
                    min_remaining: vals.get(1).copied().unwrap_or(0).max(0) as u32,
                    day_limit: tenant.requests_per_day.max(0) as u32,
                    day_remaining: vals.get(2).copied().unwrap_or(0).max(0) as u32,
                    reset_seconds: vals.get(3).copied().unwrap_or(60).max(0) as u64,
                },
                -1 => RateLimitDecision::MinuteExceeded {
                    limit: tenant.rate_limit_per_min.max(0) as u32,
                    reset_seconds: vals.get(2).copied().unwrap_or(60).max(1) as u64,
                },
                -2 => RateLimitDecision::DailyQuotaExceeded {
                    limit: tenant.requests_per_day.max(0) as u32,
                },
                _ => RateLimitDecision::Bypassed,
            },
            Ok(_) => RateLimitDecision::Bypassed,
            Err(err) => {
                tracing::warn!(error = %err, "quota redis script failed; fail-open");
                RateLimitDecision::Bypassed
            }
        }
    }

    #[allow(dead_code)]
    pub async fn try_consume_daily_quota(&self, tenant: &TenantContext) -> bool {
        self.check_rate_limit(tenant).await.is_allowed()
    }
}

async fn flush_to_jetstream(js: &jetstream::Context, batch: &mut Vec<UsageEvent>) {
    let now = Utc::now();
    for evt in batch.drain(..) {
        let contract_event = ApiUsageRequestedEvent {
            user_id: evt.user_id,
            api_key_id: evt.api_key_id,
            endpoint: evt.endpoint,
            method: evt.method,
            status_code: evt.status_code,
            response_ms: evt.response_ms,
            requested_at: now,
        };

        if let Ok(payload) = serde_json::to_vec(&contract_event) {
            if let Err(err) = js
                .publish(
                    atlsd_eventbus::subjects::USAGE_API_REQUESTED_V1.to_string(),
                    payload.into(),
                )
                .await
            {
                tracing::warn!(error = %err, "failed to publish usage telemetry event to NATS");
            }
        }
    }
}

fn seconds_until_next_utc_day() -> i64 {
    let now = Utc::now();
    let next = (now.date_naive() + chrono::Duration::days(1))
        .and_hms_opt(0, 0, 0)
        .expect("valid midnight");
    (next - now.naive_utc()).num_seconds().max(1)
}
