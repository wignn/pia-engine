mod config;

pub use config::{volatility_disable, volatility_setup, volatility_status};

use poise::serenity_prelude as serenity;

const VOLATILITY_COLOR: serenity::Colour = serenity::Colour::from_rgb(255, 215, 0);
