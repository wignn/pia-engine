use crate::repository::{
    CalendarRepository, DbPool, ForexRepository, StockRepository, TwitterRepository,
    VolatilityRepository,
};
use futures_util::{SinkExt, StreamExt};
use poise::serenity_prelude::{ChannelId, CreateEmbed, CreateEmbedFooter, CreateMessage, Http};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::Message};

const RECONNECT_DELAY_BASE: u64 = 5;
const RECONNECT_DELAY_MAX: u64 = 300;

// --- Event data structures ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreEvent {
    pub event: String,
    pub data: Option<serde_json::Value>,
    pub channel: Option<String>,
    pub timestamp: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArticleData {
    pub id: String,
    #[serde(alias = "original_title")]
    pub title: String,
    #[serde(alias = "translated_title")]
    pub title_id: Option<String>,
    pub summary: Option<String>,
    pub summary_id: Option<String>,
    pub source_name: String,
    #[serde(alias = "url")]
    pub original_url: String,
    pub sentiment: Option<String>,
    pub impact_level: Option<String>,
    #[serde(default)]
    pub currency_pairs: Vec<String>,
    pub published_at: Option<String>,
    pub processed_at: Option<String>,
    pub image_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiscordEmbed {
    pub title: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub color: Option<u32>,
    pub fields: Option<Vec<EmbedField>>,
    pub thumbnail: Option<EmbedMedia>,
    pub image: Option<EmbedMedia>,
    pub footer: Option<EmbedFooter>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmbedField {
    pub name: String,
    pub value: String,
    #[serde(default)]
    pub inline: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmbedMedia {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmbedFooter {
    pub text: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CalendarEventData {
    pub event_id: String,
    pub title: String,
    pub currency: String,
    pub date_wib: String,
    pub impact: String,
    pub forecast: String,
    pub previous: String,
    pub minutes_until: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TweetData {
    pub id: String,
    pub text: String,
    pub author_username: String,
    pub author_name: String,
    pub author_avatar: Option<String>,
    pub created_at: Option<String>,
    pub url: String,
    #[serde(default)]
    pub media_urls: Vec<String>,
}

// --- Core WebSocket Service ---

pub struct CoreWsService {
    db: DbPool,
    http: Arc<Http>,
    core_url: String,
    bot_id: String,
}

impl CoreWsService {
    pub fn new(db: DbPool, http: Arc<Http>, core_url: String, bot_id: String) -> Self {
        Self { db, http, core_url, bot_id }
    }

    pub async fn start(self: Arc<Self>) {
        println!("[CORE-WS] Starting unified WebSocket service...");
        let mut reconnect_delay = RECONNECT_DELAY_BASE;

        loop {
            match self.connect_and_listen().await {
                Ok(_) => {
                    println!("[CORE-WS] Connection closed normally");
                    reconnect_delay = RECONNECT_DELAY_BASE;
                }
                Err(e) => {
                    println!("[CORE-WS] Connection error: {}", e);
                }
            }
            println!("[CORE-WS] Reconnecting in {} seconds...", reconnect_delay);
            tokio::time::sleep(Duration::from_secs(reconnect_delay)).await;
            reconnect_delay = (reconnect_delay * 2).min(RECONNECT_DELAY_MAX);
        }
    }

    async fn connect_and_listen(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "{}/api/v1/ws/market?bot_id={}&channels=all",
            self.core_url, self.bot_id
        );
        println!("[CORE-WS] Connecting to: {}", url);

        let (ws_stream, _) = connect_async(&url).await?;
        let (mut write, mut read) = ws_stream.split();
        println!("[OK] Core WebSocket connected!");

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
                                println!("[CORE-WS] Error handling message: {}", e);
                            }
                        }
                        Some(Ok(Message::Close(_))) => {
                            println!("[CORE-WS] Server closed connection");
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

    async fn handle_message(&self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let event: CoreEvent = serde_json::from_str(text)?;

        match event.event.as_str() {
            // Forex news
            "news.new" | "news.high_impact" => {
                self.handle_news_event(&event).await?;
            }
            // Stock/equity news
            "stock.news.new" | "stock.news.high_impact" | "equity.news.new" => {
                self.handle_stock_news_event(&event).await?;
            }
            // Calendar reminders
            "calendar.reminder" => {
                self.handle_calendar_event(&event).await?;
            }
            // Gold volatility spike
            "gold.volatility_spike" => {
                self.handle_volatility_spike(&event).await?;
            }
            // X/Twitter posts
            "twitter.new" | "x.new" => {
                self.handle_twitter_event(&event).await?;
            }
            // Market price ticks — CRITICAL for /price, /prices, /alert
            "market.trade" => {
                self.handle_market_trade(text).await;
            }
            // System events
            "connected" | "subscribed" | "heartbeat" => {}
            _ => {
                println!("[CORE-WS] Unknown event: {}", event.event);
            }
        }
        Ok(())
    }

    // --- Market trade: update price cache + check alerts ---
    async fn handle_market_trade(&self, text: &str) {
        if let Ok(trade_event) = serde_json::from_str::<crate::services::market_ws::MarketTradeEvent>(text) {
            if let Some(data) = trade_event.data {
                crate::services::market_ws::update_price(&data);
                crate::services::price_alert::check_price(
                    &data.symbol, data.price, &data.price_str, &data.asset_type,
                    &self.http, &self.db,
                ).await;
            }
        }
    }

    // --- Forex news ---
    async fn handle_news_event(&self, event: &CoreEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let data = event.data.as_ref().ok_or("No data in event")?;
        let article: ArticleData = serde_json::from_value(data.get("article").cloned().ok_or("No article")?)?;
        let discord_embed: DiscordEmbed = serde_json::from_value(data.get("discord_embed").cloned().ok_or("No embed")?)?;

        if ForexRepository::is_news_sent(&self.db, &article.id).await? {
            return Ok(());
        }
        let channels = ForexRepository::get_active_channels(&self.db).await?;
        if channels.is_empty() { return Ok(()); }

        let embed = self.build_embed(&discord_embed);
        let is_high_impact = event.event == "news.high_impact";
        let mention = data.get("mention_everyone").and_then(|v| v.as_bool()).unwrap_or(false);

        for channel in &channels {
            let channel_id = ChannelId::new(channel.channel_id as u64);
            let mut message = CreateMessage::new().embed(embed.clone());
            if is_high_impact && mention {
                message = message.content("@everyone **HIGH IMPACT NEWS**");
            }
            if let Err(e) = channel_id.send_message(&self.http, message).await {
                println!("[CORE-WS] Failed to send news to {}: {}", channel.channel_id, e);
            }
        }
        ForexRepository::insert_news(&self.db, &article.id, &article.source_name).await?;
        println!("[CORE-WS] Sent forex news to {} channels: {}", channels.len(), article.title);
        Ok(())
    }

    // --- Stock/equity news ---
    async fn handle_stock_news_event(&self, event: &CoreEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let data = event.data.as_ref().ok_or("No data")?;
        let article: ArticleData = serde_json::from_value(data.get("article").cloned().ok_or("No article")?)?;
        let discord_embed: DiscordEmbed = serde_json::from_value(data.get("discord_embed").cloned().ok_or("No embed")?)?;

        if StockRepository::is_stock_news_sent(&self.db, &article.id).await? {
            return Ok(());
        }
        let channels = StockRepository::get_active_channels(&self.db).await?;
        if channels.is_empty() { return Ok(()); }

        let embed = self.build_embed(&discord_embed);
        let is_high_impact = event.event == "stock.news.high_impact";

        for channel in &channels {
            let channel_id = ChannelId::new(channel.channel_id as u64);
            let mut message = CreateMessage::new().embed(embed.clone());
            if is_high_impact && channel.mention_everyone {
                message = message.content("@everyone **BERITA SAHAM PENTING**");
            }
            if let Err(e) = channel_id.send_message(&self.http, message).await {
                println!("[CORE-WS] Failed to send stock news to {}: {}", channel.channel_id, e);
            }
        }
        StockRepository::insert_stock_news(&self.db, &article.id, &article.source_name).await?;
        println!("[CORE-WS] Sent stock news to {} channels: {}", channels.len(), article.title);
        Ok(())
    }

    // --- Calendar reminder ---
    async fn handle_calendar_event(&self, event: &CoreEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let data = event.data.as_ref().ok_or("No data")?;
        let cal: CalendarEventData = serde_json::from_value(data.get("calendar_event").cloned().ok_or("No calendar_event")?)?;

        if CalendarRepository::is_event_sent(&self.db, &cal.event_id).await? {
            return Ok(());
        }
        let channels = CalendarRepository::get_active_channels(&self.db).await?;
        if channels.is_empty() { return Ok(()); }

        let embed = CreateEmbed::new()
            .title("CALENDAR REMINDER")
            .description(format!("**{} - {}**", cal.currency, cal.title))
            .field("Waktu", &cal.date_wib, true)
            .field("Forecast", &cal.forecast, true)
            .field("Previous", &cal.previous, true)
            .field("Status", format!("High impact event starting in {} minutes", cal.minutes_until), false)
            .color(0xDC3545)
            .footer(CreateEmbedFooter::new("Fio"))
            .timestamp(poise::serenity_prelude::Timestamp::now());

        for channel in &channels {
            let channel_id = ChannelId::new(channel.channel_id as u64);
            let mut message = CreateMessage::new().embed(embed.clone());
            if channel.mention_everyone {
                message = message.content("@everyone **HIGH IMPACT EVENT**");
            }
            if let Err(e) = channel_id.send_message(&self.http, message).await {
                println!("[CORE-WS] Failed to send calendar to {}: {}", channel.channel_id, e);
            }
        }
        CalendarRepository::insert_event(&self.db, &cal.event_id, &cal.title).await?;
        println!("[CORE-WS] Sent calendar reminder to {} channels: {}", channels.len(), cal.title);
        Ok(())
    }

    // --- Volatility spike ---
    async fn handle_volatility_spike(&self, event: &CoreEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let data = event.data.as_ref().ok_or("No data")?;
        let discord_embed: DiscordEmbed = serde_json::from_value(data.get("discord_embed").cloned().ok_or("No embed")?)?;

        let channels = VolatilityRepository::get_active_channels(&self.db).await?;
        if channels.is_empty() { return Ok(()); }

        let mut embed = self.build_embed(&discord_embed);
        embed = embed.timestamp(poise::serenity_prelude::Timestamp::now());

        for channel in &channels {
            let channel_id = ChannelId::new(channel.channel_id as u64);
            let message = CreateMessage::new()
                .content("@everyone **GOLD VOLATILITY SPIKE**")
                .embed(embed.clone());
            if let Err(e) = channel_id.send_message(&self.http, message).await {
                println!("[CORE-WS] Failed to send volatility to {}: {}", channel.channel_id, e);
            }
        }
        println!("[CORE-WS] Sent gold volatility alert to {} channels", channels.len());
        Ok(())
    }

    // --- X/Twitter ---
    async fn handle_twitter_event(&self, event: &CoreEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let data = event.data.as_ref().ok_or("No data")?;
        let discord_embed: DiscordEmbed = serde_json::from_value(data.get("discord_embed").cloned().ok_or("No embed")?)?;
        let tweet: TweetData = serde_json::from_value(data.get("tweet").or_else(|| data.get("post")).cloned().ok_or("No tweet/post")?)?;

        if TwitterRepository::is_tweet_sent(&self.db, &tweet.id).await? {
            return Ok(());
        }
        let channels = TwitterRepository::get_active_channels(&self.db).await?;
        if channels.is_empty() { return Ok(()); }

        let mut embed = self.build_embed(&discord_embed);
        embed = embed.timestamp(poise::serenity_prelude::Timestamp::now());

        for channel in &channels {
            let channel_id = ChannelId::new(channel.channel_id as u64);
            let message = CreateMessage::new().embed(embed.clone());
            if let Err(e) = channel_id.send_message(&self.http, message).await {
                println!("[CORE-WS] Failed to send tweet to {}: {}", channel.channel_id, e);
            }
        }
        TwitterRepository::insert_tweet(&self.db, &tweet.id, &tweet.author_username).await?;
        println!("[CORE-WS] Sent tweet to {} channels: @{}", channels.len(), tweet.author_username);
        Ok(())
    }

    // --- Helper: build serenity embed from core's discord_embed JSON ---
    fn build_embed(&self, de: &DiscordEmbed) -> CreateEmbed {
        let mut embed = CreateEmbed::new();
        if let Some(t) = &de.title { embed = embed.title(t); }
        if let Some(d) = &de.description { embed = embed.description(d); }
        if let Some(u) = &de.url { embed = embed.url(u); }
        if let Some(c) = de.color { embed = embed.color(c); }
        if let Some(fields) = &de.fields {
            for f in fields { embed = embed.field(&f.name, &f.value, f.inline); }
        }
        if let Some(th) = &de.thumbnail { embed = embed.thumbnail(&th.url); }
        if let Some(img) = &de.image { embed = embed.image(&img.url); }
        if let Some(ft) = &de.footer {
            embed = embed.footer(CreateEmbedFooter::new(&ft.text));
        }
        embed
    }
}

pub fn start_core_ws_service(db: DbPool, http: Arc<Http>, core_url: String, bot_id: String) {
    let service = Arc::new(CoreWsService::new(db, http, core_url, bot_id));
    tokio::spawn(async move { service.start().await; });
}
