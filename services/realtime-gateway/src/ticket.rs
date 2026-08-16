use std::collections::HashMap;
use std::time::Duration;

use tokio::sync::RwLock;
use tracing::warn;
use uuid::Uuid;

pub enum TicketStore {
    Redis(redis::Client),
    Memory(RwLock<HashMap<String, MemoryTicket>>),
}

pub struct MemoryTicket {
    pub api_key: String,
    pub expires_at: std::time::Instant,
}

impl TicketStore {
    pub fn memory() -> Self {
        TicketStore::Memory(RwLock::new(HashMap::new()))
    }

    pub fn redis(client: redis::Client) -> Self {
        TicketStore::Redis(client)
    }

    fn ticket_key(ticket_id: &str) -> String {
        format!("ws-ticket:{ticket_id}")
    }

    pub async fn issue(&self, api_key: String, ttl: Duration) -> String {
        let ticket_id = Uuid::new_v4().to_string();
        match self {
            TicketStore::Redis(client) => {
                if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
                    let result: redis::RedisResult<()> = redis::cmd("SET")
                        .arg(Self::ticket_key(&ticket_id))
                        .arg(&api_key)
                        .arg("EX")
                        .arg(ttl.as_secs().max(1))
                        .query_async(&mut conn)
                        .await;
                    if let Err(err) = result {
                        warn!(error = %err, "failed to store ws ticket in redis");
                    }
                } else {
                    warn!("ws ticket issued without redis: connection failed");
                }
            }
            TicketStore::Memory(store) => {
                let mut guard = store.write().await;
                let now = std::time::Instant::now();
                guard.retain(|_, ticket| ticket.expires_at > now);
                guard.insert(
                    ticket_id.clone(),
                    MemoryTicket {
                        api_key,
                        expires_at: now + ttl,
                    },
                );
            }
        }
        ticket_id
    }

    /// Single-use redemption: returns the API key exactly once.
    pub async fn redeem(&self, ticket_id: &str) -> Option<String> {
        match self {
            TicketStore::Redis(client) => {
                let conn = client.get_multiplexed_async_connection().await.ok()?;
                let mut conn = conn;
                match redis::cmd("GETDEL")
                    .arg(Self::ticket_key(ticket_id))
                    .query_async::<Option<String>>(&mut conn)
                    .await
                {
                    Ok(value) => value,
                    Err(err) => {
                        warn!(error = %err, "failed to redeem ws ticket from redis");
                        None
                    }
                }
            }
            TicketStore::Memory(store) => {
                let ticket = store.write().await.remove(ticket_id)?;
                if std::time::Instant::now() < ticket.expires_at {
                    Some(ticket.api_key)
                } else {
                    None
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn memory_tickets_are_single_use() {
        let store = TicketStore::memory();
        let ticket = store
            .issue("api-key".to_string(), Duration::from_secs(30))
            .await;

        assert_eq!(store.redeem(&ticket).await.as_deref(), Some("api-key"));
        assert_eq!(store.redeem(&ticket).await, None, "second redeem fails");
    }

    #[tokio::test]
    async fn memory_tickets_expire() {
        let store = TicketStore::memory();
        let ticket = store
            .issue("api-key".to_string(), Duration::from_secs(0))
            .await;
        tokio::time::sleep(Duration::from_millis(10)).await;

        assert_eq!(store.redeem(&ticket).await, None, "expired ticket fails");
    }

    #[tokio::test]
    async fn unknown_ticket_redeems_to_none() {
        let store = TicketStore::memory();
        assert_eq!(store.redeem("does-not-exist").await, None);
    }
}
