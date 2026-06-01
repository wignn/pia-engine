mod embed;
mod handlers;
mod types;

use crate::repository::DbPool;
use futures_util::{SinkExt, StreamExt};
use poise::serenity_prelude::Http;
use std::sync::Arc;
use std::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub use types::*;

const RECONNECT_DELAY_BASE: u64 = 5;
const RECONNECT_DELAY_MAX: u64 = 300;

pub struct RealtimeWsService {
    pub(super) db: DbPool,
    pub(super) http: Arc<Http>,
    realtime_url: String,
    bot_id: String,
}

impl RealtimeWsService {
    pub fn new(db: DbPool, http: Arc<Http>, realtime_url: String, bot_id: String) -> Self {
        Self {
            db,
            http,
            realtime_url,
            bot_id,
        }
    }

    pub async fn start(self: Arc<Self>) {
        println!("[REALTIME-WS] Starting unified WebSocket service...");
        let mut reconnect_delay = RECONNECT_DELAY_BASE;

        loop {
            match self.connect_and_listen().await {
                Ok(_) => {
                    println!("[REALTIME-WS] Connection closed normally");
                    reconnect_delay = RECONNECT_DELAY_BASE;
                }
                Err(e) => {
                    println!("[REALTIME-WS] Connection error: {}", e);
                }
            }

            println!(
                "[REALTIME-WS] Reconnecting in {} seconds...",
                reconnect_delay
            );
            tokio::time::sleep(Duration::from_secs(reconnect_delay)).await;
            reconnect_delay = (reconnect_delay * 2).min(RECONNECT_DELAY_MAX);
        }
    }

    async fn connect_and_listen(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "{}/api/v1/ws?bot_id={}&channels=all",
            self.realtime_url, self.bot_id
        );
        println!("[REALTIME-WS] Connecting to: {}", url);

        let (ws_stream, _) = connect_async(&url).await?;
        let (mut write, mut read) = ws_stream.split();
        println!("[OK] Realtime WebSocket connected!");

        let mut heartbeat_interval = tokio::time::interval(Duration::from_secs(30));

        loop {
            tokio::select! {
                _ = heartbeat_interval.tick() => {
                    let hb = serde_json::json!({"event": "heartbeat", "data": {}});
                    write.send(Message::Text(hb.to_string())).await?;
                }
                msg = read.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            if let Err(e) = self.handle_message(&text).await {
                                println!("[REALTIME-WS] Error handling message: {}", e);
                            }
                        }
                        Some(Ok(Message::Close(_))) => {
                            println!("[REALTIME-WS] Server closed connection");
                            break;
                        }
                        Some(Ok(Message::Ping(data))) => {
                            write.send(Message::Pong(data)).await?;
                        }
                        Some(Err(e)) => return Err(Box::new(e)),
                        None => break,
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }
}

pub fn start_realtime_ws_service(
    db: DbPool,
    http: Arc<Http>,
    realtime_url: String,
    bot_id: String,
) {
    let service = Arc::new(RealtimeWsService::new(db, http, realtime_url, bot_id));
    tokio::spawn(async move {
        service.start().await;
    });
}
