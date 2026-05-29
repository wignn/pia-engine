use std::{
    collections::BTreeMap,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::get};
use serde::Serialize;
use tokio::{net::TcpListener, sync::RwLock};
use tracing::{error, info};

#[derive(Debug, Clone)]
pub struct HealthRegistry {
    inner: Arc<RwLock<BTreeMap<&'static str, WorkerHealth>>>,
    stale_after_ms: i64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct WorkerHealth {
    pub enabled: bool,
    pub connected: bool,
    pub last_tick_at_ms: Option<i64>,
    pub last_publish_at_ms: Option<i64>,
    pub received: u64,
    pub queued: u64,
    pub published: u64,
    pub dropped: u64,
    pub publish_failures: u64,
    pub publish_timeouts: u64,
    pub reconnects: u64,
    pub queue_depth: usize,
    pub queue_capacity: usize,
    pub last_disconnect_reason: Option<&'static str>,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    service: &'static str,
    status: &'static str,
    now_ms: i64,
    stale_after_ms: i64,
    workers: BTreeMap<&'static str, WorkerHealth>,
}

impl HealthRegistry {
    pub fn new(stale_after_ms: i64) -> Self {
        Self {
            inner: Arc::new(RwLock::new(BTreeMap::new())),
            stale_after_ms,
        }
    }

    pub async fn register(&self, worker: &'static str, enabled: bool) {
        let mut inner = self.inner.write().await;
        inner.entry(worker).or_default().enabled = enabled;
    }

    pub async fn set_queue_capacity(&self, worker: &'static str, capacity: usize) {
        self.update(worker, |state| state.queue_capacity = capacity)
            .await;
    }

    pub async fn set_connected(&self, worker: &'static str, connected: bool) {
        self.update(worker, |state| state.connected = connected)
            .await;
    }

    pub async fn record_tick(&self, worker: &'static str) {
        let now = now_ms();
        self.update(worker, |state| {
            state.received = state.received.saturating_add(1);
            state.last_tick_at_ms = Some(now);
        })
        .await;
    }

    pub async fn record_queued(&self, worker: &'static str, queue_depth: usize) {
        self.update(worker, |state| {
            state.queued = state.queued.saturating_add(1);
            state.queue_depth = queue_depth;
        })
        .await;
    }

    pub async fn record_published(&self, worker: &'static str, queue_depth: usize) {
        let now = now_ms();
        self.update(worker, |state| {
            state.published = state.published.saturating_add(1);
            state.last_publish_at_ms = Some(now);
            state.queue_depth = queue_depth;
        })
        .await;
    }

    pub async fn record_drop(&self, worker: &'static str, queue_depth: usize) {
        self.update(worker, |state| {
            state.dropped = state.dropped.saturating_add(1);
            state.queue_depth = queue_depth;
        })
        .await;
    }

    pub async fn record_publish_failure(&self, worker: &'static str, queue_depth: usize) {
        self.update(worker, |state| {
            state.publish_failures = state.publish_failures.saturating_add(1);
            state.queue_depth = queue_depth;
        })
        .await;
    }

    pub async fn record_publish_timeout(&self, worker: &'static str, queue_depth: usize) {
        self.update(worker, |state| {
            state.publish_timeouts = state.publish_timeouts.saturating_add(1);
            state.queue_depth = queue_depth;
        })
        .await;
    }

    pub async fn record_disconnect(&self, worker: &'static str, reason: &'static str) {
        self.update(worker, |state| {
            state.connected = false;
            state.reconnects = state.reconnects.saturating_add(1);
            state.last_disconnect_reason = Some(reason);
        })
        .await;
    }

    async fn update(&self, worker: &'static str, update: impl FnOnce(&mut WorkerHealth)) {
        let mut inner = self.inner.write().await;
        let state = inner.entry(worker).or_default();
        update(state);
    }

    async fn snapshot(&self) -> HealthResponse {
        HealthResponse {
            service: "ingestion-gateway",
            status: self.status().await,
            now_ms: now_ms(),
            stale_after_ms: self.stale_after_ms,
            workers: self.inner.read().await.clone(),
        }
    }

    async fn status(&self) -> &'static str {
        let now = now_ms();
        let inner = self.inner.read().await;
        let unhealthy = inner.values().any(|worker| {
            worker.enabled
                && (!worker.connected
                    || worker
                        .last_tick_at_ms
                        .map(|last| now.saturating_sub(last) > self.stale_after_ms)
                        .unwrap_or(true))
        });

        if unhealthy { "degraded" } else { "healthy" }
    }
}

pub async fn serve(bind_addr: String, registry: HealthRegistry) {
    let app = Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .with_state(registry);

    match TcpListener::bind(&bind_addr).await {
        Ok(listener) => {
            info!(bind_addr = %bind_addr, "ingestion health server running");
            if let Err(err) = axum::serve(listener, app).await {
                error!(error = %err, "ingestion health server failed");
            }
        }
        Err(err) => {
            error!(error = %err, bind_addr = %bind_addr, "failed to bind ingestion health server")
        }
    }
}

async fn health(State(registry): State<HealthRegistry>) -> Json<HealthResponse> {
    Json(registry.snapshot().await)
}

async fn ready(State(registry): State<HealthRegistry>) -> impl IntoResponse {
    let snapshot = registry.snapshot().await;
    if snapshot.status == "healthy" {
        (StatusCode::OK, Json(snapshot))
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(snapshot))
    }
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default()
}
