mod calendar;
mod config;

pub use calendar::forex_calendar;
pub use config::{forex_news_disable, forex_news_enable, forex_news_setup, forex_news_status};

const FOREX_COLOR: poise::serenity_prelude::Colour =
    poise::serenity_prelude::Colour::from_rgb(0, 150, 136);
