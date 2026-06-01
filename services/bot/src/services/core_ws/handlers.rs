use crate::repository::{
    CalendarRepository, ForexRepository, StockRepository, TwitterRepository, VolatilityRepository,
};
use poise::serenity_prelude::{ChannelId, CreateEmbed, CreateEmbedFooter, CreateMessage};

use super::RealtimeWsService;
use super::embed::build_embed;
use super::types::{ArticleData, CalendarEventData, CoreEvent, DiscordEmbed, TweetData};

impl RealtimeWsService {
    pub(super) async fn handle_message(
        &self,
        text: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let event: CoreEvent = serde_json::from_str(text)?;

        match event.event.as_str() {
            "news.new" | "news.high_impact" => self.handle_news_event(&event).await?,
            "stock.news.new" | "stock.news.high_impact" | "equity.news.new" => {
                self.handle_stock_news_event(&event).await?;
            }
            "calendar.reminder" => self.handle_calendar_event(&event).await?,
            "gold.volatility_spike" => self.handle_volatility_spike(&event).await?,
            "market.alert" => self.handle_alert(&event).await?,
            "twitter.new" | "x.new" => self.handle_twitter_event(&event).await?,
            "market.trade" => self.handle_market_trade(text).await,
            "connected" | "subscribed" | "heartbeat" => {}
            _ => println!("[REALTIME-WS] Unknown event: {}", event.event),
        }

        Ok(())
    }

    async fn handle_market_trade(&self, text: &str) {
        if let Ok(trade_event) =
            serde_json::from_str::<crate::services::market_ws::MarketTradeEvent>(text)
            && let Some(wrapper) = trade_event.data
        {
            let cached = crate::services::market_ws::update_price(&wrapper.tick);
            crate::services::price_alert::check_price(
                &cached.symbol,
                cached.price,
                &cached.price_str,
                &cached.asset_type,
                &self.http,
                &self.db,
            )
            .await;
        }
    }

    async fn handle_news_event(
        &self,
        event: &CoreEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let data = event.data.as_ref().ok_or("No data in event")?;
        let article: ArticleData =
            serde_json::from_value(data.get("article").cloned().ok_or("No article")?)?;
        let discord_embed: DiscordEmbed =
            serde_json::from_value(data.get("discord_embed").cloned().ok_or("No embed")?)?;

        if ForexRepository::is_news_sent(&self.db, &article.id).await? {
            return Ok(());
        }
        let channels = ForexRepository::get_active_channels(&self.db).await?;
        if channels.is_empty() {
            return Ok(());
        }

        let embed = build_embed(&discord_embed);
        let is_high_impact = event.event == "news.high_impact";
        let mention = data
            .get("mention_everyone")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        for channel in &channels {
            let channel_id = ChannelId::new(channel.channel_id as u64);
            let mut message = CreateMessage::new().embed(embed.clone());
            if is_high_impact && mention {
                message = message.content("@everyone **HIGH IMPACT NEWS**");
            }
            if let Err(e) = channel_id.send_message(&self.http, message).await {
                println!(
                    "[REALTIME-WS] Failed to send news to {}: {}",
                    channel.channel_id, e
                );
            }
        }

        ForexRepository::insert_news(&self.db, &article.id, &article.source_name).await?;
        println!(
            "[REALTIME-WS] Sent forex news to {} channels: {}",
            channels.len(),
            article.title
        );
        Ok(())
    }

    async fn handle_stock_news_event(
        &self,
        event: &CoreEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let data = event.data.as_ref().ok_or("No data")?;
        let article: ArticleData =
            serde_json::from_value(data.get("article").cloned().ok_or("No article")?)?;
        let discord_embed: DiscordEmbed =
            serde_json::from_value(data.get("discord_embed").cloned().ok_or("No embed")?)?;

        if StockRepository::is_stock_news_sent(&self.db, &article.id).await? {
            return Ok(());
        }
        let channels = StockRepository::get_active_channels(&self.db).await?;
        if channels.is_empty() {
            return Ok(());
        }

        let embed = build_embed(&discord_embed);
        let is_high_impact = event.event == "stock.news.high_impact";

        for channel in &channels {
            let channel_id = ChannelId::new(channel.channel_id as u64);
            let mut message = CreateMessage::new().embed(embed.clone());
            if is_high_impact && channel.mention_everyone {
                message = message.content("@everyone **BERITA SAHAM PENTING**");
            }
            if let Err(e) = channel_id.send_message(&self.http, message).await {
                println!(
                    "[REALTIME-WS] Failed to send stock news to {}: {}",
                    channel.channel_id, e
                );
            }
        }

        StockRepository::insert_stock_news(&self.db, &article.id, &article.source_name).await?;
        println!(
            "[REALTIME-WS] Sent stock news to {} channels: {}",
            channels.len(),
            article.title
        );
        Ok(())
    }

    async fn handle_calendar_event(
        &self,
        event: &CoreEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let data = event.data.as_ref().ok_or("No data")?;
        let cal: CalendarEventData = serde_json::from_value(
            data.get("calendar_event")
                .cloned()
                .ok_or("No calendar_event")?,
        )?;

        if CalendarRepository::is_event_sent(&self.db, &cal.event_id).await? {
            return Ok(());
        }
        let channels = CalendarRepository::get_active_channels(&self.db).await?;
        if channels.is_empty() {
            return Ok(());
        }

        let embed = CreateEmbed::new()
            .title("CALENDAR REMINDER")
            .description(format!("**{} - {}**", cal.currency, cal.title))
            .field("Waktu", &cal.date_wib, true)
            .field("Forecast", &cal.forecast, true)
            .field("Previous", &cal.previous, true)
            .field(
                "Status",
                format!(
                    "High impact event starting in {} minutes",
                    cal.minutes_until
                ),
                false,
            )
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
                println!(
                    "[REALTIME-WS] Failed to send calendar to {}: {}",
                    channel.channel_id, e
                );
            }
        }

        CalendarRepository::insert_event(&self.db, &cal.event_id, &cal.title).await?;
        println!(
            "[REALTIME-WS] Sent calendar reminder to {} channels: {}",
            channels.len(),
            cal.title
        );
        Ok(())
    }

    async fn handle_volatility_spike(
        &self,
        event: &CoreEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let data = event.data.as_ref().ok_or("No data")?;
        let discord_embed: DiscordEmbed =
            serde_json::from_value(data.get("discord_embed").cloned().ok_or("No embed")?)?;

        let channels = VolatilityRepository::get_active_channels(&self.db).await?;
        if channels.is_empty() {
            return Ok(());
        }

        let embed =
            build_embed(&discord_embed).timestamp(poise::serenity_prelude::Timestamp::now());

        for channel in &channels {
            let channel_id = ChannelId::new(channel.channel_id as u64);
            let message = CreateMessage::new()
                .content("@everyone **GOLD VOLATILITY SPIKE**")
                .embed(embed.clone());
            if let Err(e) = channel_id.send_message(&self.http, message).await {
                println!(
                    "[REALTIME-WS] Failed to send volatility to {}: {}",
                    channel.channel_id, e
                );
            }
        }

        println!(
            "[REALTIME-WS] Sent gold volatility alert to {} channels",
            channels.len()
        );
        Ok(())
    }

    async fn handle_alert(
        &self,
        event: &CoreEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let data = event.data.as_ref().ok_or("No data")?;
        let discord_embed: DiscordEmbed =
            serde_json::from_value(data.get("discord_embed").cloned().ok_or("No embed")?)?;

        let channels = VolatilityRepository::get_active_channels(&self.db).await?;
        if channels.is_empty() {
            return Ok(());
        }

        let embed =
            build_embed(&discord_embed).timestamp(poise::serenity_prelude::Timestamp::now());
        for channel in &channels {
            let channel_id = ChannelId::new(channel.channel_id as u64);
            let message = CreateMessage::new()
                .content("@everyone **MARKET ALERT**")
                .embed(embed.clone());
            if let Err(e) = channel_id.send_message(&self.http, message).await {
                println!(
                    "[REALTIME-WS] Failed to send alert to {}: {}",
                    channel.channel_id, e
                );
            }
        }

        println!(
            "[REALTIME-WS] Sent market alert to {} channels",
            channels.len()
        );
        Ok(())
    }

    async fn handle_twitter_event(
        &self,
        event: &CoreEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let data = event.data.as_ref().ok_or("No data")?;
        let discord_embed: DiscordEmbed =
            serde_json::from_value(data.get("discord_embed").cloned().ok_or("No embed")?)?;
        let tweet: TweetData = serde_json::from_value(
            data.get("tweet")
                .or_else(|| data.get("post"))
                .cloned()
                .ok_or("No tweet/post")?,
        )?;

        if TwitterRepository::is_tweet_sent(&self.db, &tweet.id).await? {
            return Ok(());
        }
        let channels = TwitterRepository::get_active_channels(&self.db).await?;
        if channels.is_empty() {
            return Ok(());
        }

        let embed =
            build_embed(&discord_embed).timestamp(poise::serenity_prelude::Timestamp::now());

        for channel in &channels {
            let channel_id = ChannelId::new(channel.channel_id as u64);
            let message = CreateMessage::new().embed(embed.clone());
            if let Err(e) = channel_id.send_message(&self.http, message).await {
                println!(
                    "[REALTIME-WS] Failed to send tweet to {}: {}",
                    channel.channel_id, e
                );
            }
        }

        TwitterRepository::insert_tweet(&self.db, &tweet.id, &tweet.author_username).await?;
        println!(
            "[REALTIME-WS] Sent tweet to {} channels: @{}",
            channels.len(),
            tweet.author_username
        );
        Ok(())
    }
}
