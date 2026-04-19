use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::{mpsc, RwLock};
use tracing::{error, info};

use super::collector;

type ClientId = u64;

struct StatsClient {
    id: ClientId,
    sender: mpsc::Sender<Vec<u8>>,
}

/// WebSocket hub for broadcasting system stats at regular intervals.
pub struct StatsHub {
    clients: Arc<RwLock<HashMap<ClientId, StatsClient>>>,
    next_id: Arc<RwLock<u64>>,
}

impl StatsHub {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(RwLock::new(1)),
        })
    }

    /// Run the periodic stats broadcast loop.
    pub async fn run(self: Arc<Self>, interval: Duration, mut shutdown: tokio::sync::watch::Receiver<bool>) {
        let mut ticker = tokio::time::interval(interval);

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    self.broadcast().await;
                }
                _ = shutdown.changed() => {
                    let mut clients = self.clients.write().await;
                    clients.clear();
                    info!("stats hub stopped");
                    return;
                }
            }
        }
    }

    async fn broadcast(&self) {
        let payload = collector::collect();
        let data = match serde_json::to_vec(&payload) {
            Ok(d) => d,
            Err(e) => {
                error!(error = %e, "stats marshal failed");
                return;
            }
        };

        let clients = self.clients.read().await;
        for client in clients.values() {
            let _ = client.sender.try_send(data.clone());
        }
    }

    /// Handle a new WebSocket connection for stats.
    pub async fn handle_socket(self: Arc<Self>, socket: WebSocket) {
        let mut next = self.next_id.write().await;
        let id = *next;
        *next += 1;
        drop(next);

        let (tx, mut rx) = mpsc::channel::<Vec<u8>>(16);
        self.clients.write().await.insert(id, StatsClient { id, sender: tx });

        let count = self.clients.read().await.len();
        info!(total = count, "stats ws client connected");

        // Send immediate snapshot
        let snapshot = serde_json::to_vec(&collector::collect()).unwrap_or_default();
        if let Some(client) = self.clients.read().await.get(&id) {
            let _ = client.sender.try_send(snapshot);
        }

        let (mut ws_tx, mut ws_rx) = socket.split();

        let hub_write = self.clone();
        let write_task = tokio::spawn(async move {
            let mut ping_interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                tokio::select! {
                    Some(msg) = rx.recv() => {
                        let text = String::from_utf8_lossy(&msg).into_owned();
                        if ws_tx.send(Message::Text(text.into())).await.is_err() {
                            break;
                        }
                    }
                    _ = ping_interval.tick() => {
                        if ws_tx.send(Message::Ping(vec![].into())).await.is_err() {
                            break;
                        }
                    }
                    else => break,
                }
            }
            hub_write.clients.write().await.remove(&id);
        });

        let read_task = tokio::spawn(async move {
            let timeout = Duration::from_secs(120);
            loop {
                match tokio::time::timeout(timeout, ws_rx.next()).await {
                    Ok(Some(Ok(Message::Close(_)))) | Ok(None) | Err(_) => break,
                    Ok(Some(Err(_))) => break,
                    _ => {}
                }
            }
        });

        tokio::select! {
            _ = write_task => {}
            _ = read_task => {}
        }

        self.clients.write().await.remove(&id);
        let count = self.clients.read().await.len();
        info!(total = count, "stats ws client disconnected");
    }
}
