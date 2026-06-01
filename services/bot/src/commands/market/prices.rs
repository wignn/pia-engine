use crate::commands::{Context, Error};
use crate::services::market_ws;

use super::autocomplete::autocomplete_symbol;
use super::embeds;

#[poise::command(prefix_command, slash_command, rename = "price")]
pub async fn price(
    ctx: Context<'_>,
    #[description = "Symbol to check (e.g. XAUUSD, BTCUSDT)"]
    #[autocomplete = "autocomplete_symbol"]
    symbol: String,
) -> Result<(), Error> {
    let symbol = symbol.to_uppercase();
    let embed = match market_ws::get_price(&symbol) {
        Some(cached) => embeds::price(&symbol, &cached),
        None => embeds::symbol_not_found(&symbol, &available_symbols()),
    };

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Show all current market prices
#[poise::command(prefix_command, slash_command, rename = "prices")]
pub async fn prices(ctx: Context<'_>) -> Result<(), Error> {
    let all = market_ws::get_all_prices();
    let embed = if all.is_empty() {
        embeds::no_market_data()
    } else {
        embeds::prices(&all)
    };

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

pub fn available_symbols() -> Vec<String> {
    market_ws::get_all_prices()
        .iter()
        .map(|price| price.symbol.clone())
        .collect()
}
