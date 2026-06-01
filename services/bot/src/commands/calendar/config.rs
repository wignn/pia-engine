use crate::commands::feed_embed;
use crate::commands::{Context, Error};
use crate::repository::CalendarRepository;
use poise::serenity_prelude as serenity;
use serenity::{CreateEmbed, CreateEmbedFooter, Timestamp};

use super::CALENDAR_COLOR;

#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "ADMINISTRATOR"
)]
pub async fn calendar_setup(
    ctx: Context<'_>,
    #[description = "Channel for calendar reminders"] channel: serenity::GuildChannel,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Must be used in a guild")?.get();
    let channel_id = channel.id.get();

    let pool = ctx.data().db.as_ref();
    CalendarRepository::insert_channel(pool, guild_id, channel_id).await?;

    let embed = CreateEmbed::default()
        .title("Calendar Reminder Setup Complete")
        .description(format!(
            "High-impact economic event reminders will be sent to <#{}>\n\n\
            **Event Types:**\n\
            Central Bank Decisions, NFP, CPI, GDP, Interest Rate\n\n\
            **Timing:**\n\
            Reminders sent 15 minutes before each event\n\n\
            **Timezone:**\n\
            All times displayed in WIB (UTC+7)",
            channel_id
        ))
        .color(CALENDAR_COLOR)
        .footer(CreateEmbedFooter::new("Fio Calendar"))
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
pub async fn calendar_disable(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Must be used in a guild")?.get();

    let pool = ctx.data().db.as_ref();
    CalendarRepository::disable_channel(pool, guild_id).await?;

    let embed = feed_embed::disabled(
        "Calendar Reminders Disabled",
        "/calendar_setup",
        "Calendar reminder",
    );

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "ADMINISTRATOR"
)]
pub async fn calendar_enable(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Must be used in a guild")?.get();

    let pool = ctx.data().db.as_ref();
    CalendarRepository::enable_channel(pool, guild_id).await?;

    let embed = feed_embed::enabled(
        "Calendar Reminders Enabled",
        "Calendar reminder notifications have been re-enabled.",
        CALENDAR_COLOR,
    );

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
