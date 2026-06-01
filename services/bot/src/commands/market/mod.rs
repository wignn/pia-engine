mod alerts;
mod autocomplete;
mod embeds;
mod prices;

pub use alerts::{alert, alert_remove, alerts};
pub use prices::{price, prices};

const MAX_ALERTS_PER_USER: i64 = 10;
