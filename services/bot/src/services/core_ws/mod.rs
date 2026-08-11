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
    api_key: String,
}

impl RealtimeWsService {
    pub fn new(db: DbPool, http: Arc<Http>, realtime_url: String, api_key: String) -> Self {
        Self {
            db,
            http,
            realtime_url,
            api_key,
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
        let mut url = reqwest::Url::parse(&self.realtime_url)?;
        url.query_pairs_mut().append_pair("api_key", &self.api_key);
        println!("[REALTIME-WS] Connecting to realtime gateway");

        let (ws_stream, _) = connect_async(url.as_str()).await?;
        let (mut write, mut read) = ws_stream.split();
        println!("[OK] Realtime WebSocket connected!");

        let subscription = serde_json::json!({
            "action": "subscribe",
            "channels": ["market_prices", "news_feed"]
        });
        write
            .send(Message::Text(subscription.to_string().into()))
            .await?;

        loop {
            tokio::select! {
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
    api_key: String,
) {
    let service = Arc::new(RealtimeWsService::new(db, http, realtime_url, api_key));
    tokio::spawn(async move {
        service.start().await;
    });
}
