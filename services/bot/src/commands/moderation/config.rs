use crate::commands::{Context, Error};
use crate::repository::ModerationRepository;
use poise::serenity_prelude as serenity;
use serenity::{Colour, CreateEmbed, Mentionable, Timestamp};

#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "ADMINISTRATOR"
)]
pub async fn autorole_set(
    ctx: Context<'_>,
    #[description = "Role to assign to new members"] role: serenity::Role,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Must be used in a guild")?;
    let pool = ctx.data().db.as_ref();
    ModerationRepository::set_auto_role(pool, guild_id.get(), role.id.get()).await?;

    let embed = CreateEmbed::new()
        .title("Auto-Role Set")
        .description(format!(
            "New members will automatically receive the {} role.",
            role.mention()
        ))
        .color(Colour::DARK_GREEN)
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
pub async fn autorole_disable(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Must be used in a guild")?;
    let pool = ctx.data().db.as_ref();
    ModerationRepository::disable_auto_role(pool, guild_id.get()).await?;

    let embed = CreateEmbed::new()
        .title("Auto-Role Disabled")
        .description("New members will no longer receive an automatic role.")
        .color(Colour::RED)
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
pub async fn log_setup(
    ctx: Context<'_>,
    #[description = "Channel for logging"] channel: serenity::GuildChannel,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Must be used in a guild")?;
    let pool = ctx.data().db.as_ref();
    ModerationRepository::set_log_channel(pool, guild_id.get(), channel.id.get()).await?;

    let embed = CreateEmbed::new()
        .title("Logging Enabled")
        .description(format!(
            "Member join/leave events will be logged to {}.",
            channel.mention()
        ))
        .color(Colour::DARK_GREEN)
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
pub async fn log_disable(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Must be used in a guild")?;
    let pool = ctx.data().db.as_ref();
    ModerationRepository::disable_logging(pool, guild_id.get()).await?;

    let embed = CreateEmbed::new()
        .title("Logging Disabled")
        .description("Member join/leave logging has been disabled.")
        .color(Colour::RED)
        .timestamp(Timestamp::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
