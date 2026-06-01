use super::{Context, Error};
use crate::repository::StockRepository;
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter, Timestamp};

/// Subscribe this channel to equity/stock news alerts
#[poise::command(slash_command, required_permissions = "MANAGE_CHANNELS")]
pub async fn subscribe(
    ctx: Context<'_>,
    #[description = "Mention @everyone for high impact news"] mention_everyone: Option<bool>,
) -> Result<(), Error> {
    let pool = ctx.data().db.as_ref();
    let channel_id = ctx.channel_id().get();
    let guild_id = ctx.guild_id().map(|g| g.get()).unwrap_or(0);
    let mention = mention_everyone.unwrap_or(false);

    sqlx::query("UPDATE stock_channels SET is_active = 0 WHERE guild_id = ? AND is_active = 1")
        .bind(guild_id as i64)
        .execute(pool)
        .await?;

    sqlx::query(
        "INSERT INTO stock_channels (channel_id, guild_id, mention_everyone, is_active)
         VALUES (?, ?, ?, 1)
         ON CONFLICT (channel_id) DO UPDATE SET mention_everyone = excluded.mention_everyone, is_active = 1",
    )
    .bind(channel_id as i64)
    .bind(guild_id as i64)
    .bind(mention)
    .execute(pool)
    .await?;

    let embed = CreateEmbed::new()
        .title("Stock News Alert Aktif")
        .description("Channel ini sekarang menerima alert berita saham Indonesia.")
        .field(
            "Sumber",
            "CNBC Indonesia, Kontan, Bisnis Indonesia, Detik Finance, IDX Channel",
            false,
        )
        .field(
            "Mention Everyone",
            if mention {
                "Ya (untuk high impact)"
            } else {
                "Tidak"
            },
            true,
        )
        .color(0x00FF00)
        .footer(CreateEmbedFooter::new(
            "Gunakan /stocknews unsubscribe untuk berhenti",
        ))
        .timestamp(Timestamp::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Unsubscribe this channel from stock news alerts
#[poise::command(slash_command, required_permissions = "MANAGE_CHANNELS")]
pub async fn unsubscribe(ctx: Context<'_>) -> Result<(), Error> {
    let pool = ctx.data().db.as_ref();
    let channel_id = ctx.channel_id().get();

    StockRepository::disable_channel(pool, channel_id).await?;

    let embed = CreateEmbed::new()
        .title("Stock News Alert Dinonaktifkan")
        .description("Channel ini tidak akan menerima alert berita saham lagi.")
        .color(0xFF6600)
        .timestamp(Timestamp::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Check stock news subscription status for this channel
#[poise::command(slash_command)]
pub async fn status(ctx: Context<'_>) -> Result<(), Error> {
    let pool = ctx.data().db.as_ref();
    let channel_id = ctx.channel_id().get();

    let channel = StockRepository::get_channel(pool, channel_id).await?;

    let embed = match channel {
        Some(ch) if ch.is_active => CreateEmbed::new()
            .title("Stock News Alert Status")
            .field("Status", "✅ ==Aktif", true)
            .field(
                "Mention Everyone",
                if ch.mention_everyone { "Ya" } else { "Tidak" },
                true,
            )
            .color(0x00FF00)
            .timestamp(Timestamp::now()),
        _ => CreateEmbed::new()
            .title("Stock News Alert Status")
            .description("Channel ini tidak berlangganan stock news alert.")
            .field("Aktifkan", "Gunakan `/stocknews subscribe`", false)
            .color(0x808080)
            .timestamp(Timestamp::now()),
    };

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
