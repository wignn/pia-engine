mod config;
mod latest;
mod placeholders;

pub use config::{status, subscribe, unsubscribe};
pub use latest::latest;
pub use placeholders::{market, search};

use super::{Context, Error};

const STOCK_COLOR: u32 = 0x5865F2;
const STOCK_ACTION_COLOR: u32 = 0x2962FF;

#[poise::command(
    slash_command,
    subcommands("subscribe", "unsubscribe", "status", "latest"),
    subcommand_required
)]
pub async fn stocknews(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}
