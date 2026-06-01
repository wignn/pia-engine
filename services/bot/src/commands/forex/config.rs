use crate::commands::feed_embed;
use crate::commands::{Context, Error};
use crate::repository::ForexRepository;
use poise::serenity_prelude as serenity;
use serenity::{CreateEmbed, CreateEmbedFooter, Timestamp};

use super::FOREX_COLOR;

#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "ADMINISTRATOR"
)]
pub async fn forex_news_setup(
    ctx: Context<'_>,
    #[description = "Channel for forex news"] channel: serenity::GuildChannel,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Must be used in a guild")?.get();
    let channel_id = channel.id.get();

    let pool = ctx.data().db.as_ref();
    ForexRepository::insert_channel(pool, guild_id, channel_id).await?;

    let embed = CreateEmbed::default()
        .title("Forex News Setup Complete")
        .description(format!(
            "Real-time forex news will be sent to <#{}>\n\n\
            **Coverage:**\n\
            `USD` `EUR` `GBP` `JPY` `CHF` `AUD` `NZD` `CAD`\n\n\
            **Sources:**\n\
            FXStreet, Forex Factory, Investing.com\n\n\
            **Impact Levels:**\n\
            `HIGH` - Central bank decisions, NFP, CPI, GDP\n\
            `MEDIUM` - Trade balance, PMI, Housing data",
            channel_id
        ))
        .color(FOREX_COLOR)
        .footer(CreateEmbedFooter::new("Updates every 60 seconds"))
        .timestamp(Timestamp::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "ADMINISTRATOR"
)]
pub async fn forex_news_disable(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Must be used in a guild")?.get();

    let pool = ctx.data().db.as_ref();
    ForexRepository::disable_channel(pool, guild_id).await?;

    let embed = feed_embed::disabled("Forex News Disabled", "/forex_news_setup", "Forex news");

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Enable forex news notifications
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "ADMINISTRATOR"
)]
pub async fn forex_news_enable(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Must be used in a guild")?.get();

    let pool = ctx.data().db.as_ref();
    ForexRepository::enable_channel(pool, guild_id).await?;

    let embed = feed_embed::enabled(
        "Forex News Enabled",
        "Forex news notifications have been re-enabled.",
        FOREX_COLOR,
    );

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Check forex news status
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn forex_news_status(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Must be used in a guild")?.get();

    let pool = ctx.data().db.as_ref();
    let channel = ForexRepository::get_channel(pool, guild_id).await?;

    let embed = feed_embed::status(
        "Forex News Status",
        "/forex_news_setup",
        channel.map(|ch| (ch.channel_id, ch.is_active)),
        FOREX_COLOR,
        &[],
    );

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
