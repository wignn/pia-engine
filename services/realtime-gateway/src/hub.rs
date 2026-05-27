use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{client::ClientHandle, streams};

pub type ClientId = u64;

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

    pub async fn register(
        &self,
        bot_id: String,
        streams: HashSet<String>,
        user_id: Option<Uuid>,
    ) -> (ClientId, mpsc::Receiver<Vec<u8>>) {
        let mut next = self.next_id.write().await;
        let id = *next;
        *next += 1;

        let (tx, rx) = mpsc::channel::<Vec<u8>>(256);

        let handle = ClientHandle {
            id,
            bot_id: bot_id.clone(),
            user_id,
            streams,
            sender: tx,
        };

        self.clients.write().await.insert(id, handle);
        let count = self.clients.read().await.len();
        info!(bot_id = %bot_id, user_id = ?user_id, total = count, "ws client connected");

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

    pub async fn unregister(&self, id: ClientId) {
        let removed = self.clients.write().await.remove(&id);
        if let Some(client) = removed {
            let count = self.clients.read().await.len();
            info!(bot_id = %client.bot_id, total = count, "ws client disconnected");
        }
    }

    pub async fn broadcast(
        &self,
        event_type: &str,
        data: serde_json::Value,
        channel: &str,
    ) -> usize {
        let stream = streams::event_stream(channel, &data);
        let msg = json!({
            "event": event_type,
            "data": data,
            "channel": channel,
            "stream": stream,
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

        let candidate_streams = streams::candidate_streams(channel, &data);

        for client in clients.values() {
            if candidate_streams.is_disjoint(&client.streams) {
                continue;
            }

            if let Ok(()) = client.sender.try_send(payload.clone()) {
                count += 1;
            }
        }

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

        if channel == "market_data" {
            debug!(
                event = event_type,
                channel = channel,
                clients = count,
                "broadcast sent"
            );
        } else {
            info!(
                event = event_type,
                channel = channel,
                clients = count,
                "broadcast sent"
            );
        }
        count
    }

    pub async fn subscribe(&self, id: ClientId, streams: HashSet<String>) -> bool {
        let mut clients = self.clients.write().await;
        let Some(client) = clients.get_mut(&id) else {
            return false;
        };
        client.streams.extend(streams);
        true
    }

    pub async fn unsubscribe(&self, id: ClientId, streams: &HashSet<String>) -> bool {
        let mut clients = self.clients.write().await;
        let Some(client) = clients.get_mut(&id) else {
            return false;
        };
        client.streams.retain(|stream| !streams.contains(stream));
        true
    }

    pub async fn list_subscriptions(&self, id: ClientId) -> Option<Vec<String>> {
        let clients = self.clients.read().await;
        let mut streams: Vec<String> = clients.get(&id)?.streams.iter().cloned().collect();
        streams.sort();
        Some(streams)
    }

    /// Get the number of connected clients.
    #[allow(dead_code)]
    pub async fn client_count(&self) -> usize {
        self.clients.read().await.len()
    }

    #[allow(dead_code)]
    pub async fn user_connection_count(&self, user_id: &Uuid) -> usize {
        let clients = self.clients.read().await;
        clients
            .values()
            .filter(|c| c.user_id.as_ref() == Some(user_id))
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn set(values: &[&str]) -> HashSet<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    async fn recv_json(rx: &mut mpsc::Receiver<Vec<u8>>) -> Value {
        let bytes = rx.recv().await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn register_sends_welcome_and_tracks_client_count() {
        let hub = Hub::new(None, "test".to_string());
        let user_id = Uuid::new_v4();
        let (_id, mut rx) = hub
            .register("bot-1".to_string(), set(&["forex_news"]), Some(user_id))
            .await;

        assert_eq!(hub.client_count().await, 1);
        assert_eq!(hub.user_connection_count(&user_id).await, 1);

        let welcome = recv_json(&mut rx).await;
        assert_eq!(welcome["event"], "connected");
        assert_eq!(welcome["data"]["bot_id"], "bot-1");
    }

    #[tokio::test]
    async fn unregister_removes_client_and_user_count() {
        let hub = Hub::new(None, "test".to_string());
        let user_id = Uuid::new_v4();
        let (id, _rx) = hub
            .register("bot-1".to_string(), set(&["forex_news"]), Some(user_id))
            .await;

        hub.unregister(id).await;

        assert_eq!(hub.client_count().await, 0);
        assert_eq!(hub.user_connection_count(&user_id).await, 0);
    }

    #[tokio::test]
    async fn broadcast_respects_channel_subscriptions() {
        let hub = Hub::new(None, "test".to_string());
        let (_forex_id, mut forex_rx) = hub
            .register("forex-bot".to_string(), set(&["forex_news"]), None)
            .await;
        let (_stock_id, mut stock_rx) = hub
            .register("stock-bot".to_string(), set(&["stock_news"]), None)
            .await;
        recv_json(&mut forex_rx).await;
        recv_json(&mut stock_rx).await;

        let sent = hub
            .broadcast("forex.new", json!({ "title": "EUR/USD" }), "forex_news")
            .await;

        assert_eq!(sent, 1);
        assert_eq!(recv_json(&mut forex_rx).await["event"], "forex.new");
        assert!(stock_rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn broadcast_all_subscription_receives_any_channel() {
        let hub = Hub::new(None, "test".to_string());
        let (_id, mut rx) = hub
            .register("all-bot".to_string(), set(&["all"]), None)
            .await;
        recv_json(&mut rx).await;

        let sent = hub
            .broadcast("stock.new", json!({ "title": "AAPL" }), "stock_news")
            .await;

        assert_eq!(sent, 1);
        assert_eq!(recv_json(&mut rx).await["channel"], "stock_news");
    }

    #[tokio::test]
    async fn market_data_filter_matches_symbols() {
        let hub = Hub::new(None, "test".to_string());
        let (_aapl_id, mut aapl_rx) = hub
            .register("aapl-bot".to_string(), set(&["market_data:AAPL"]), None)
            .await;
        let (_msft_id, mut msft_rx) = hub
            .register("msft-bot".to_string(), set(&["market_data:MSFT"]), None)
            .await;
        recv_json(&mut aapl_rx).await;
        recv_json(&mut msft_rx).await;

        let sent = hub
            .broadcast(
                "market.tick",
                json!({ "tick": { "symbol": "AAPL", "price": 100 } }),
                "market_data",
            )
            .await;

        assert_eq!(sent, 1);
        assert_eq!(recv_json(&mut aapl_rx).await["event"], "market.tick");
        assert!(msft_rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn x_filter_matches_author_username_case_insensitively() {
        let hub = Hub::new(None, "test".to_string());
        let (_matching_id, mut matching_rx) = hub
            .register("matching-bot".to_string(), set(&["x:federalreserve"]), None)
            .await;
        let (_other_id, mut other_rx) = hub
            .register("other-bot".to_string(), set(&["x:ecb"]), None)
            .await;
        recv_json(&mut matching_rx).await;
        recv_json(&mut other_rx).await;

        let sent = hub
            .broadcast(
                "x.post",
                json!({ "post": { "author_username": "FederalReserve" } }),
                "x",
            )
            .await;

        assert_eq!(sent, 1);
        assert_eq!(recv_json(&mut matching_rx).await["event"], "x.post");
        assert!(other_rx.try_recv().is_err());
    }
}
