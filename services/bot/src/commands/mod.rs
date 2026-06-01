pub mod admin;
pub mod calendar;
mod feed_embed;
pub mod forex;
pub mod general;
pub mod market;
pub mod moderation;
pub mod ping;
pub mod stock;
pub mod sys;
pub mod twitter;
pub mod volatility;

use crate::repository::DbPool;
use poise::serenity_prelude::UserId;
use std::collections::HashSet;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(Clone)]
pub struct Data {
    pub owners: HashSet<UserId>,
    pub db: DbPool,
    pub api_http_url: String,
}

pub fn all() -> Vec<poise::Command<Data, Error>> {
    vec![
        ping::ping(),
        general::ping(),
        general::say(),
        general::purge(),
        admin::everyone(),
        sys::sys(),
        moderation::warn(),
        moderation::warnings(),
        moderation::clearwarnings(),
        moderation::mute(),
        moderation::unmute(),
        moderation::kick(),
        moderation::ban(),
        moderation::unban(),
        moderation::autorole_set(),
        moderation::autorole_disable(),
        moderation::log_setup(),
        moderation::log_disable(),
        forex::forex_news_setup(),
        forex::forex_news_disable(),
        forex::forex_news_enable(),
        forex::forex_news_status(),
        forex::forex_calendar(),
        calendar::calendar_setup(),
        calendar::calendar_disable(),
        calendar::calendar_enable(),
        calendar::calendar_status(),
        calendar::calendar_mention(),
        stock::stocknews(),
        market::price(),
        market::prices(),
        market::alert(),
        market::alerts(),
        market::alert_remove(),
        volatility::volatility_setup(),
        volatility::volatility_disable(),
        volatility::volatility_status(),
        twitter::twitter_setup(),
        twitter::twitter_disable(),
        twitter::twitter_enable(),
        twitter::twitter_status(),
    ]
}

impl std::fmt::Debug for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Data")
            .field("owners", &self.owners)
            .field("db", &"Arc<SqlitePool>")
            .field("api_http_url", &self.api_http_url)
            .finish()
    }
}
