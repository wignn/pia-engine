use crate::commands::feed_embed;
use crate::commands::{Context, Error};
use crate::repository::CalendarRepository;
use poise::serenity_prelude::{CreateEmbed, Timestamp};

use super::CALENDAR_COLOR;

#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn calendar_status(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Must be used in a guild")?.get();

    let pool = ctx.data().db.as_ref();
    let channel = CalendarRepository::get_channel(pool, guild_id).await?;

    let embed = match channel {
        Some(ch) => {
            let mention = if ch.mention_everyone { "Yes" } else { "No" };
            feed_embed::status(
                "Calendar Reminder Status",
                "/calendar_setup",
                Some((ch.channel_id, ch.is_active)),
                CALENDAR_COLOR,
                &[("Mention Everyone", mention, true)],
            )
        }
        None => feed_embed::status(
            "Calendar Reminder Status",
            "/calendar_setup",
            None,
            CALENDAR_COLOR,
            &[],
        ),
    };

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "ADMINISTRATOR"
)]
pub async fn calendar_mention(
    ctx: Context<'_>,
    #[description = "Enable @everyone mention"] enable: bool,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Must be used in a guild")?.get();

    let pool = ctx.data().db.as_ref();
    CalendarRepository::set_mention_everyone(pool, guild_id, enable).await?;

    let status = if enable { "enabled" } else { "disabled" };
    let embed = CreateEmbed::default()
        .title("Mention Setting Updated")
        .description(format!(
            "@everyone mention for high-impact events has been {}.",
            status
        ))
        .color(CALENDAR_COLOR)
        .timestamp(Timestamp::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
