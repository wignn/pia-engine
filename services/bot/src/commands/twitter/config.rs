use super::TWITTER_COLOR;
use crate::commands::feed_embed;
use crate::commands::{Context, Error};
use crate::repository::TwitterRepository;
use poise::serenity_prelude as serenity;
use serenity::{CreateEmbed, CreateEmbedFooter, Timestamp};

#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "ADMINISTRATOR"
)]
pub async fn twitter_setup(
    ctx: Context<'_>,
    #[description = "Channel for X/Twitter feed"] channel: serenity::GuildChannel,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Must be used in a guild")?.get();
    let channel_id = channel.id.get();

    let pool = ctx.data().db.as_ref();
    TwitterRepository::insert_channel(pool, guild_id, channel_id).await?;

    let embed = CreateEmbed::default()
        .title("X/Twitter Feed Setup Complete")
        .description(format!(
            "Real-time X/Twitter feed will be sent to <#{}>\n\n\
            **How it works:**\n\
            Tweets from configured accounts will be posted here automatically.\n\n\
            **Note:**\n\
            Accounts to follow are configured server-side via `X_USERNAMES` env var.",
            channel_id
        ))
        .color(TWITTER_COLOR)
        .footer(CreateEmbedFooter::new("X/Twitter Feed"))
        .timestamp(Timestamp::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Disable X/Twitter feed notifications
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "ADMINISTRATOR"
)]
pub async fn twitter_disable(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Must be used in a guild")?.get();

    let pool = ctx.data().db.as_ref();
    TwitterRepository::disable_channel(pool, guild_id).await?;

    let embed = feed_embed::disabled(
        "X/Twitter Feed Disabled",
        "/twitter_setup",
        "X/Twitter feed",
    );

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Enable X/Twitter feed notifications
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "ADMINISTRATOR"
)]
pub async fn twitter_enable(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Must be used in a guild")?.get();

    let pool = ctx.data().db.as_ref();
    TwitterRepository::enable_channel(pool, guild_id).await?;

    let embed = feed_embed::enabled(
        "X/Twitter Feed Enabled",
        "X/Twitter feed notifications have been re-enabled.",
        TWITTER_COLOR,
    );

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Check X/Twitter feed status
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn twitter_status(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Must be used in a guild")?.get();

    let pool = ctx.data().db.as_ref();
    let channel = TwitterRepository::get_channel(pool, guild_id).await?;

    let embed = feed_embed::status(
        "X/Twitter Feed Status",
        "/twitter_setup",
        channel.map(|ch| (ch.channel_id, ch.is_active)),
        TWITTER_COLOR,
        &[],
    );

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
