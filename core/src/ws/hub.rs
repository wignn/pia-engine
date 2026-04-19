use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{info, error, warn};
use serde_json::json;
use uuid::Uuid;

use super::client::ClientHandle;

pub type ClientId = u64;

/// Central WebSocket hub managing client connections and channel-based broadcasting.
/// Enhanced with tenant-aware filtering for X feed and market data.
pub struct Hub {
    clients: Arc<RwLock<HashMap<ClientId, ClientHandle>>>,
    next_id: Arc<RwLock<u64>>,
    redis_client: Option<redis::Client>,
    redis_channel_prefix: String,
}

impl Hub {
    pub fn new(redis_client: Option<redis::Client>, redis_channel_prefix: String) -> Arc<Self> {
        Arc::new(Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(RwLock::new(1)),
            redis_client,
            redis_channel_prefix,
        })
    }

    /// Register a new client and return its ID + sender channel.
    pub async fn register(
        &self,
        bot_id: String,
        channels: HashSet<String>,
        user_id: Option<Uuid>,
        x_usernames: HashSet<String>,
        tv_symbols: HashSet<String>,
    ) -> (ClientId, mpsc::Receiver<Vec<u8>>) {
        let mut next = self.next_id.write().await;
        let id = *next;
        *next += 1;

        let (tx, rx) = mpsc::channel::<Vec<u8>>(256);

        let handle = ClientHandle {
            id,
            bot_id: bot_id.clone(),
            user_id,
            channels,
            x_usernames,
            tv_symbols,
            sender: tx,
        };

        self.clients.write().await.insert(id, handle);
        let count = self.clients.read().await.len();
        info!(bot_id = %bot_id, user_id = ?user_id, total = count, "ws client connected");

        // Send welcome message
        let welcome = serde_json::to_vec(&json!({
            "event": "connected",
            "data": {
                "message": "Connected to World Info WebSocket",
                "bot_id": bot_id,
            }
        }))
        .unwrap_or_default();

        if let Some(client) = self.clients.read().await.get(&id) {
            let _ = client.sender.try_send(welcome);
        }

        (id, rx)
    }

    /// Unregister a client by ID.
    pub async fn unregister(&self, id: ClientId) {
        let removed = self.clients.write().await.remove(&id);
        if let Some(client) = removed {
            let count = self.clients.read().await.len();
            info!(bot_id = %client.bot_id, total = count, "ws client disconnected");
        }
    }

    /// Broadcast a message to all clients subscribed to the given channel.
    /// For "x" channel: filters by client's x_usernames set.
    /// For "market_data" channel: filters by client's tv_symbols set.
    pub async fn broadcast(&self, event_type: &str, data: serde_json::Value, channel: &str) -> usize {
        let msg = json!({
            "event": event_type,
            "data": data,
            "channel": channel,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        let payload = match serde_json::to_vec(&msg) {
            Ok(p) => p,
            Err(e) => {
                error!(error = %e, "failed to marshal broadcast");
                return 0;
            }
        };

        let clients = self.clients.read().await;
        let mut count = 0;

        for client in clients.values() {
            // Channel check
            if !client.channels.contains("all") && !client.channels.contains(channel) {
                continue;
            }

            // Tenant-specific filtering for X feed
            if channel == "x" && !client.x_usernames.is_empty() {
                let author = data.get("post")
                    .and_then(|p| p.get("author_username"))
                    .and_then(|a| a.as_str())
                    .unwrap_or("")
                    .to_lowercase();
                if !author.is_empty() && !client.x_usernames.contains(&author) {
                    continue;
                }
            }

            // Tenant-specific filtering for market data
            if channel == "market_data" && !client.tv_symbols.is_empty() {
                let symbol = data.get("tick")
                    .and_then(|t| t.get("symbol"))
                    .and_then(|s| s.as_str())
                    .unwrap_or("");
                if !symbol.is_empty() && !client.tv_symbols.contains(symbol) {
                    continue;
                }
            }

            match client.sender.try_send(payload.clone()) {
                Ok(()) => count += 1,
                Err(_) => {} // client send buffer full, skip
            }
        }

        // Redis fanout
        if let Some(redis_client) = &self.redis_client {
            let payload_text = String::from_utf8_lossy(&payload).to_string();
            let redis_channel = format!("{}:{}", self.redis_channel_prefix, channel);

            match redis_client.get_multiplexed_async_connection().await {
                Ok(mut conn) => {
                    let publish_result: redis::RedisResult<i64> = redis::cmd("PUBLISH")
                        .arg(&redis_channel)
                        .arg(&payload_text)
                        .query_async(&mut conn)
                        .await;

                    if let Err(e) = publish_result {
                        warn!(error = %e, channel = %redis_channel, "redis publish failed");
                    }
                }
                Err(e) => {
                    warn!(error = %e, "redis connection failed during publish");
                }
            }
        }

        info!(event = event_type, channel = channel, clients = count, "broadcast sent");
        count
    }

    /// Get the number of connected clients.
    pub async fn client_count(&self) -> usize {
        self.clients.read().await.len()
    }

    /// Count WS connections for a specific user.
    pub async fn user_connection_count(&self, user_id: &Uuid) -> usize {
        let clients = self.clients.read().await;
        clients.values()
            .filter(|c| c.user_id.as_ref() == Some(user_id))
            .count()
    }
}
