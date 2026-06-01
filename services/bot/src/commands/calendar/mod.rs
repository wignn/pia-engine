mod config;
mod status;

pub use config::{calendar_disable, calendar_enable, calendar_setup};
pub use status::{calendar_mention, calendar_status};

const CALENDAR_COLOR: poise::serenity_prelude::Colour =
    poise::serenity_prelude::Colour::from_rgb(220, 53, 69);
