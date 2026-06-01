mod config;

pub use config::{twitter_disable, twitter_enable, twitter_setup, twitter_status};

use poise::serenity_prelude as serenity;

const TWITTER_COLOR: serenity::Colour = serenity::Colour::from_rgb(29, 161, 242);
