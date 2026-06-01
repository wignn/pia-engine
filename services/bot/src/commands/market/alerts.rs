use crate::commands::{Context, Error};
use crate::repository::PriceAlertRepository;
use crate::services::{market_ws, price_alert};

use super::MAX_ALERTS_PER_USER;
use super::autocomplete::autocomplete_symbol;
use super::embeds;
use super::prices::available_symbols;

#[poise::command(slash_command, rename = "market_alert")]
pub async fn alert(
    ctx: Context<'_>,
    #[description = "Symbol (e.g. XAUUSD, BTCUSDT)"]
    #[autocomplete = "autocomplete_symbol"]
    symbol: String,
    #[description = "Target price"] target_price: f64,
) -> Result<(), Error> {
    let symbol = symbol.to_uppercase();
    let db = &ctx.data().db;
    let user_id = ctx.author().id.get();
    let guild_id = ctx.guild_id().map(|g| g.get()).unwrap_or(0);

    let Some(current) = market_ws::get_price(&symbol) else {
        let embed = embeds::symbol_not_found(&symbol, &available_symbols());
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    };

    let count = PriceAlertRepository::count_user_alerts(db, user_id).await?;
    if count >= MAX_ALERTS_PER_USER {
        ctx.send(poise::CreateReply::default().embed(embeds::alert_limit_reached(count)))
            .await?;
        return Ok(());
    }

    let Some(direction) = alert_direction(target_price, current.price) else {
        ctx.send(poise::CreateReply::default().embed(embeds::invalid_target()))
            .await?;
        return Ok(());
    };

    let created =
        PriceAlertRepository::create_alert(db, user_id, guild_id, &symbol, target_price, direction)
            .await?;
    price_alert::add_to_cache(&created);

    let embed = embeds::alert_created(&symbol, &current, target_price, direction, created.id);
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command, rename = "market_alerts")]
pub async fn alerts(ctx: Context<'_>) -> Result<(), Error> {
    let db = &ctx.data().db;
    let user_id = ctx.author().id.get();
    let user_alerts = PriceAlertRepository::get_user_alerts(db, user_id).await?;

    let embed = if user_alerts.is_empty() {
        embeds::empty_alerts()
    } else {
        embeds::alert_list(&user_alerts)
    };

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command, rename = "market_alert_remove")]
pub async fn alert_remove(
    ctx: Context<'_>,
    #[description = "Alert ID to remove"] alert_id: i64,
) -> Result<(), Error> {
    let db = &ctx.data().db;
    let user_id = ctx.author().id.get();
    let deleted = PriceAlertRepository::delete_alert(db, alert_id, user_id).await?;

    if deleted {
        price_alert::remove_from_cache(alert_id);
    }

    let embed = if deleted {
        embeds::alert_removed(alert_id)
    } else {
        embeds::alert_not_found(alert_id)
    };

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

fn alert_direction(target_price: f64, current_price: f64) -> Option<&'static str> {
    if target_price > current_price {
        Some("above")
    } else if target_price < current_price {
        Some("below")
    } else {
        None
    }
}
