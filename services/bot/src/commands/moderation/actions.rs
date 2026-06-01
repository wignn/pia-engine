use crate::commands::{Context, Error};
use crate::utils::embed;
use poise::serenity_prelude as serenity;
use serenity::{Colour, CreateEmbed, CreateEmbedFooter, Member, Mentionable, Timestamp};

use super::duration::parse_duration;

/// Timeout (mute) a user
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MODERATE_MEMBERS"
)]
pub async fn mute(
    ctx: Context<'_>,
    #[description = "User to mute"] mut user: Member,
    #[description = "Duration (e.g. 5m, 1h, 7d)"] duration: String,
    #[description = "Reason"] reason: Option<String>,
) -> Result<(), Error> {
    let reason_text = reason.unwrap_or_else(|| "No reason provided".to_string());
    let dur = parse_duration(&duration).ok_or("Invalid duration format. Use: 5m, 1h, 7d")?;

    if dur.as_secs() > 28 * 24 * 3600 {
        ctx.send(poise::CreateReply::default().embed(embed::error(
            "Invalid Duration",
            "Maximum timeout duration is 28 days.",
        )))
        .await?;
        return Ok(());
    }

    let timeout_until = serenity::Timestamp::from_unix_timestamp(
        chrono::Utc::now().timestamp() + dur.as_secs() as i64,
    )?;
    user.disable_communication_until_datetime(&ctx.http(), timeout_until)
        .await?;

    let embed = CreateEmbed::new()
        .title("User Muted")
        .description(format!(
            "**User:** {}\n**Duration:** {}\n**Reason:** {}",
            user.user.mention(),
            duration,
            reason_text
        ))
        .color(Colour::RED)
        .footer(CreateEmbedFooter::new(format!(
            "Muted by {}",
            ctx.author().name
        )))
        .timestamp(Timestamp::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MODERATE_MEMBERS"
)]
pub async fn unmute(
    ctx: Context<'_>,
    #[description = "User to unmute"] mut user: Member,
) -> Result<(), Error> {
    user.enable_communication(&ctx.http()).await?;

    let embed = CreateEmbed::new()
        .title("User Unmuted")
        .description(format!("{} can now speak again.", user.user.mention()))
        .color(Colour::DARK_GREEN)
        .footer(CreateEmbedFooter::new(format!(
            "Unmuted by {}",
            ctx.author().name
        )))
        .timestamp(Timestamp::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "KICK_MEMBERS"
)]
pub async fn kick(
    ctx: Context<'_>,
    #[description = "User to kick"] user: Member,
    #[description = "Reason"] reason: Option<String>,
) -> Result<(), Error> {
    let reason_text = reason.unwrap_or_else(|| "No reason provided".to_string());
    user.kick_with_reason(&ctx.http(), &reason_text).await?;

    let embed = CreateEmbed::new()
        .title("User Kicked")
        .description(format!(
            "**User:** {} ({})\n**Reason:** {}",
            user.user.name, user.user.id, reason_text
        ))
        .color(Colour::ORANGE)
        .footer(CreateEmbedFooter::new(format!(
            "Kicked by {}",
            ctx.author().name
        )))
        .timestamp(Timestamp::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "BAN_MEMBERS"
)]
pub async fn ban(
    ctx: Context<'_>,
    #[description = "User to ban"] user: Member,
    #[description = "Reason"] reason: Option<String>,
    #[description = "Delete message days (0-7)"] delete_days: Option<u8>,
) -> Result<(), Error> {
    let reason_text = reason.unwrap_or_else(|| "No reason provided".to_string());
    let del_days = delete_days.unwrap_or(0).min(7);

    user.ban_with_reason(&ctx.http(), del_days, &reason_text)
        .await?;

    let embed = CreateEmbed::new()
        .title("User Banned")
        .description(format!(
            "**User:** {} ({})\n**Reason:** {}\n**Messages deleted:** {} days",
            user.user.name, user.user.id, reason_text, del_days
        ))
        .color(Colour::DARK_RED)
        .footer(CreateEmbedFooter::new(format!(
            "Banned by {}",
            ctx.author().name
        )))
        .timestamp(Timestamp::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "BAN_MEMBERS"
)]
pub async fn unban(
    ctx: Context<'_>,
    #[description = "User ID to unban"] user_id: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Must be used in a guild")?;
    let uid: u64 = user_id.parse().map_err(|_| "Invalid user ID")?;
    let user_id_parsed = serenity::UserId::new(uid);

    guild_id.unban(&ctx.http(), user_id_parsed).await?;

    let embed = CreateEmbed::new()
        .title("User Unbanned")
        .description(format!("User ID `{}` has been unbanned.", uid))
        .color(Colour::DARK_GREEN)
        .footer(CreateEmbedFooter::new(format!(
            "Unbanned by {}",
            ctx.author().name
        )))
        .timestamp(Timestamp::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
