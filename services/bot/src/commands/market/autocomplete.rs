use crate::commands::Context;
use crate::services::market_ws;
use poise::serenity_prelude::AutocompleteChoice;

pub async fn autocomplete_symbol<'a>(
    _ctx: Context<'a>,
    partial: &'a str,
) -> Vec<AutocompleteChoice> {
    let partial_upper = partial.to_uppercase();

    let mut choices: Vec<(String, String)> = market_ws::get_all_prices()
        .into_iter()
        .filter(|p| p.symbol.starts_with(&partial_upper) || partial.is_empty())
        .take(25)
        .map(|p| {
            let label = format!(
                "{} ({}) - {}",
                p.symbol,
                p.asset_type.to_uppercase(),
                p.price_str,
            );
            (label, p.symbol.clone())
        })
        .collect();

    choices.sort_by(|a, b| a.1.cmp(&b.1));
    choices
        .into_iter()
        .map(|(name, value)| AutocompleteChoice::new(name, value))
        .collect()
}
