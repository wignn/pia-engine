use super::VOLATILITY_COLOR;
use crate::commands::feed_embed;
use crate::commands::{Context, Error};
use crate::repository::VolatilityRepository;
use poise::serenity_prelude as serenity;
use serenity::{CreateEmbed, CreateEmbedFooter, Timestamp};

#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "ADMINISTRATOR"
)]
pub async fn volatility_setup(
    ctx: Context<'_>,
    #[description = "Channel for volatility alerts"] channel: serenity::GuildChannel,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Must be used in a guild")?.get();
    let channel_id = channel.id.get();

    let pool = ctx.data().db.as_ref();
    VolatilityRepository::insert_channel(pool, guild_id, channel_id).await?;

    let embed = CreateEmbed::default()
        .title("Gold Volatility Alert Setup")
        .description(format!(
            "Volatility spike alerts will be sent to <#{}>\n\n\
            **How it works:**\n\
            Monitors XAUUSD ATR (Average True Range) in real-time.\n\
            When current ATR exceeds 2x the historical average,\n\
            a warning is sent to this channel.\n\n\
            **Use case:**\n\
            Detect sudden gold volatility spikes before they hit the news.",
            channel_id
        ))
        .color(VOLATILITY_COLOR)
        .footer(CreateEmbedFooter::new("Fio Volatility Detector"))
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
pub async fn volatility_disable(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Must be used in a guild")?.get();

    let pool = ctx.data().db.as_ref();
    VolatilityRepository::disable_channel(pool, guild_id).await?;

    let embed = feed_embed::disabled(
        "Volatility Alerts Disabled",
        "/volatility_setup",
        "Gold volatility spike alerts",
    );

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn volatility_status(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Must be used in a guild")?.get();

    let pool = ctx.data().db.as_ref();
    let channel = VolatilityRepository::get_channel(pool, guild_id).await?;

    let embed = feed_embed::status(
        "Gold Volatility Alert Status",
        "/volatility_setup",
        channel.map(|ch| (ch.channel_id, ch.is_active)),
        VOLATILITY_COLOR,
        &[
            ("Symbol", "XAUUSD", true),
            ("Threshold", "ATR > 2x Average", true),
        ],
    );

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
