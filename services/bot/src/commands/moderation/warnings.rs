use crate::commands::{Context, Error};
use crate::repository::ModerationRepository;
use crate::utils::embed;
use poise::serenity_prelude as serenity;
use serenity::{Colour, CreateEmbed, CreateEmbedFooter, Member, Mentionable, Timestamp};

#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MODERATE_MEMBERS"
)]
pub async fn warn(
    ctx: Context<'_>,
    #[description = "User to warn"] user: Member,
    #[description = "Reason for warning"] reason: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Must be used in a guild")?;
    let moderator = ctx.author();

    if user.user.id == moderator.id {
        ctx.send(
            poise::CreateReply::default()
                .embed(embed::error("Cannot Warn", "You cannot warn yourself!")),
        )
        .await?;
        return Ok(());
    }

    if user.user.bot {
        ctx.send(
            poise::CreateReply::default()
                .embed(embed::error("Cannot Warn", "You cannot warn bots!")),
        )
        .await?;
        return Ok(());
    }

    let pool = ctx.data().db.as_ref();
    ModerationRepository::add_warning(
        pool,
        guild_id.get(),
        user.user.id.get(),
        moderator.id.get(),
        &reason,
    )
    .await?;
    let warn_count =
        ModerationRepository::get_warning_count(pool, guild_id.get(), user.user.id.get()).await?;

    let embed = CreateEmbed::new()
        .title("User Warned")
        .description(format!(
            "**User:** {}\n**Reason:** {}\n**Total Warnings:** {}",
            user.user.mention(),
            reason,
            warn_count
        ))
        .color(Colour::ORANGE)
        .footer(CreateEmbedFooter::new(format!(
            "Warned by {}",
            moderator.name
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
pub async fn warnings(
    ctx: Context<'_>,
    #[description = "User to check"] user: Member,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Must be used in a guild")?;
    let pool = ctx.data().db.as_ref();
    let warns =
        ModerationRepository::get_warnings(pool, guild_id.get(), user.user.id.get()).await?;

    if warns.is_empty() {
        let embed = CreateEmbed::new()
            .title("No Warnings")
            .description(format!("{} has no warnings.", user.user.mention()))
            .color(Colour::DARK_GREEN)
            .timestamp(Timestamp::now());
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    }

    let warnings_list = warns
        .iter()
        .enumerate()
        .map(|(i, warning)| {
            format!(
                "**{}. ID #{}** - {}\n- <t:{}:R>",
                i + 1,
                warning.id,
                warning.reason,
                chrono::NaiveDateTime::parse_from_str(&warning.created_at, "%Y-%m-%d %H:%M:%S")
                    .map(|dt| dt.and_utc().timestamp())
                    .unwrap_or(0)
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    let embed = CreateEmbed::new()
        .title(format!("Warnings for {}", user.user.name))
        .description(warnings_list)
        .color(Colour::ORANGE)
        .footer(CreateEmbedFooter::new(format!(
            "Total: {} warnings",
            warns.len()
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
pub async fn clearwarnings(
    ctx: Context<'_>,
    #[description = "User to clear warnings"] user: Member,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Must be used in a guild")?;
    let pool = ctx.data().db.as_ref();
    let cleared =
        ModerationRepository::clear_warnings(pool, guild_id.get(), user.user.id.get()).await?;

    let embed = CreateEmbed::new()
        .title("Warnings Cleared")
        .description(format!(
            "Cleared **{}** warning(s) for {}",
            cleared,
            user.user.mention()
        ))
        .color(Colour::DARK_GREEN)
        .footer(CreateEmbedFooter::new(format!(
            "Cleared by {}",
            ctx.author().name
        )))
        .timestamp(Timestamp::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
