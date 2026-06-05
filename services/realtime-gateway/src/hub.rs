use crate::metrics::Metrics;
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
    metrics: Arc<Metrics>,
}

impl Hub {
    pub const fn connection_counter_ttl_sec() -> i64 {
        120
    }

    pub const fn connection_counter_refresh_sec() -> u64 {
        30
    }

    pub fn new(redis_client: Option<redis::Client>, redis_channel_prefix: String) -> Arc<Self> {
        Arc::new(Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(RwLock::new(1)),
            redis_client,
            redis_channel_prefix,
            metrics: Arc::new(Metrics::default()),
        })
    }

    pub fn metrics(&self) -> Arc<Metrics> {
        self.metrics.clone()
    }

    pub async fn register_api_key(
        &self,
        bot_id: String,
        streams: HashSet<String>,
        user_id: Option<Uuid>,
        api_key_id: String,
    ) -> (ClientId, mpsc::Receiver<Vec<u8>>) {
        self.register_with_key(bot_id, streams, user_id, Some(api_key_id))
            .await
    }

    async fn register_with_key(
        &self,
        bot_id: String,
        streams: HashSet<String>,
        user_id: Option<Uuid>,
        api_key_id: Option<String>,
    ) -> (ClientId, mpsc::Receiver<Vec<u8>>) {
        let mut next = self.next_id.write().await;
        let id = *next;
        *next += 1;

        let (tx, rx) = mpsc::channel::<Vec<u8>>(256);

        let handle = ClientHandle {
            id,
            bot_id: bot_id.clone(),
            user_id,
            api_key_id,
            streams,
            sender: tx,
        };

        self.clients.write().await.insert(id, handle);
        self.metrics.connection_opened();
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

    pub async fn can_register_api_key(&self, api_key_id: &str, limit: i32) -> bool {
        if limit < 1 {
            return false;
        }
        self.api_key_connection_count(api_key_id).await < limit as usize
    }

    pub async fn try_acquire_api_key_slot(&self, api_key_id: &str, limit: i32) -> bool {
        if limit < 1 {
            return false;
        }
        if let Some(redis_client) = &self.redis_client {
            let redis_key = self.api_key_connection_key(api_key_id);
            match redis_client.get_multiplexed_async_connection().await {
                Ok(mut conn) => {
                    let acquired: redis::RedisResult<i64> = redis::Script::new(
                        r#"
                        local current = tonumber(redis.call('GET', KEYS[1]) or '0')
                        local limit = tonumber(ARGV[1])
                        if current >= limit then
                            return 0
                        end
                        current = redis.call('INCR', KEYS[1])
                        redis.call('EXPIRE', KEYS[1], tonumber(ARGV[2]))
                        return 1
                        "#,
                    )
                    .key(&redis_key)
                    .arg(limit)
                    .arg(Self::connection_counter_ttl_sec())
                    .invoke_async(&mut conn)
                    .await;
                    if let Ok(value) = acquired {
                        return value == 1;
                    }
                    warn!(api_key_id = %api_key_id, "redis ws connection counter failed; falling back to local count");
                }
                Err(err) => warn!(error = %err, "redis unavailable for ws connection counter"),
            }
        }
        self.can_register_api_key(api_key_id, limit).await
    }

    pub async fn release_api_key_slot(&self, api_key_id: &str) {
        if let Some(redis_client) = &self.redis_client {
            let redis_key = self.api_key_connection_key(api_key_id);
            match redis_client.get_multiplexed_async_connection().await {
                Ok(mut conn) => {
                    let release: redis::RedisResult<i64> = redis::Script::new(
                        r#"
                        local current = tonumber(redis.call('GET', KEYS[1]) or '0')
                        if current <= 1 then
                            redis.call('DEL', KEYS[1])
                            return 0
                        end
                        return redis.call('DECR', KEYS[1])
                        "#,
                    )
                    .key(&redis_key)
                    .invoke_async(&mut conn)
                    .await;
                    if let Err(err) = release {
                        warn!(error = %err, api_key_id = %api_key_id, "redis ws connection release failed");
                    }
                }
                Err(err) => warn!(error = %err, "redis unavailable for ws connection release"),
            }
        }
    }

    pub async fn refresh_api_key_slot(&self, api_key_id: &str) {
        if let Some(redis_client) = &self.redis_client {
            let redis_key = self.api_key_connection_key(api_key_id);
            match redis_client.get_multiplexed_async_connection().await {
                Ok(mut conn) => {
                    let refresh: redis::RedisResult<bool> = redis::cmd("EXPIRE")
                        .arg(&redis_key)
                        .arg(Self::connection_counter_ttl_sec())
                        .query_async(&mut conn)
                        .await;
                    if let Err(err) = refresh {
                        warn!(error = %err, api_key_id = %api_key_id, "redis ws connection refresh failed");
                    }
                }
                Err(err) => warn!(error = %err, "redis unavailable for ws connection refresh"),
            }
        }
    }

    fn api_key_connection_key(&self, api_key_id: &str) -> String {
        format!(
            "{}:ws-connections:{}",
            self.redis_channel_prefix, api_key_id
        )
    }

    pub async fn unregister(&self, id: ClientId) {
        let removed = self.clients.write().await.remove(&id);
        if let Some(client) = removed {
            self.metrics.connection_closed();
            if let Some(api_key_id) = &client.api_key_id {
                self.release_api_key_slot(api_key_id).await;
            }
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
            } else {
                self.metrics.send_failure();
            }
        }
        self.metrics.broadcast(count);

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

    #[allow(dead_code)]
    pub async fn api_key_connection_count(&self, api_key_id: &str) -> usize {
        let clients = self.clients.read().await;
        clients
            .values()
            .filter(|c| c.api_key_id.as_deref() == Some(api_key_id))
            .count()
    }

    pub async fn client_api_key_id(&self, id: ClientId) -> Option<String> {
        self.clients.read().await.get(&id)?.api_key_id.clone()
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

    async fn register_test(
        hub: &Arc<Hub>,
        bot_id: &str,
        streams: HashSet<String>,
        user_id: Option<Uuid>,
    ) -> (ClientId, mpsc::Receiver<Vec<u8>>) {
        hub.register_api_key(
            bot_id.to_string(),
            streams,
            user_id,
            Uuid::new_v4().to_string(),
        )
        .await
    }

    #[tokio::test]
    async fn register_sends_welcome_and_tracks_client_count() {
        let hub = Hub::new(None, "test".to_string());
        let user_id = Uuid::new_v4();
        let (_id, mut rx) = register_test(&hub, "bot-1", set(&["forex_news"]), Some(user_id)).await;

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
        let (id, _rx) = register_test(&hub, "bot-1", set(&["forex_news"]), Some(user_id)).await;

        hub.unregister(id).await;

        assert_eq!(hub.client_count().await, 0);
        assert_eq!(hub.user_connection_count(&user_id).await, 0);
    }

    #[test]
    fn connection_counter_ttl_is_longer_than_ping_interval() {
        assert_eq!(Hub::connection_counter_ttl_sec(), 120);
        assert_eq!(Hub::connection_counter_refresh_sec(), 30);
        assert!(Hub::connection_counter_ttl_sec() as u64 > Hub::connection_counter_refresh_sec());
    }

    #[tokio::test]
    async fn connection_limit_allows_until_limit_and_rejects_next() {
        let hub = Hub::new(None, "test".to_string());
        let api_key_id = Uuid::new_v4().to_string();

        assert!(hub.try_acquire_api_key_slot(&api_key_id, 2).await);
        let (first_id, _first_rx) = hub
            .register_api_key(
                "bot-1".to_string(),
                set(&["market_data"]),
                None,
                api_key_id.clone(),
            )
            .await;

        assert!(hub.try_acquire_api_key_slot(&api_key_id, 2).await);
        let (second_id, _second_rx) = hub
            .register_api_key(
                "bot-2".to_string(),
                set(&["market_data"]),
                None,
                api_key_id.clone(),
            )
            .await;

        assert!(!hub.try_acquire_api_key_slot(&api_key_id, 2).await);

        hub.unregister(first_id).await;
        assert!(hub.try_acquire_api_key_slot(&api_key_id, 2).await);

        hub.unregister(second_id).await;
    }

    #[tokio::test]
    async fn broadcast_respects_channel_subscriptions() {
        let hub = Hub::new(None, "test".to_string());
        let (_forex_id, mut forex_rx) =
            register_test(&hub, "forex-bot", set(&["forex_news"]), None).await;
        let (_stock_id, mut stock_rx) =
            register_test(&hub, "stock-bot", set(&["stock_news"]), None).await;
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
        let (_id, mut rx) = register_test(&hub, "all-bot", set(&["all"]), None).await;
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
        let (_aapl_id, mut aapl_rx) =
            register_test(&hub, "aapl-bot", set(&["market_data:AAPL"]), None).await;
        let (_msft_id, mut msft_rx) =
            register_test(&hub, "msft-bot", set(&["market_data:MSFT"]), None).await;
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
        let (_matching_id, mut matching_rx) =
            register_test(&hub, "matching-bot", set(&["x:federalreserve"]), None).await;
        let (_other_id, mut other_rx) =
            register_test(&hub, "other-bot", set(&["x:ecb"]), None).await;
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
