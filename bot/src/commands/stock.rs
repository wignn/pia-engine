use crate::commands::Data;
use crate::repository::StockRepository;
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter, Timestamp};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// ---------------------------------------------------------------------------
// /stocknews <sub-command>
// ---------------------------------------------------------------------------

#[poise::command(
    slash_command,
    subcommands("subscribe", "unsubscribe", "status", "latest"),
    subcommand_required
)]
pub async fn stocknews(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

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
        .field("Sumber", "CNBC Indonesia, Kontan, Bisnis Indonesia, Detik Finance, IDX Channel", false)
        .field("Mention Everyone", if mention { "Ya (untuk high impact)" } else { "Tidak" }, true)
        .color(0x00FF00)
        .footer(CreateEmbedFooter::new("Gunakan /stocknews unsubscribe untuk berhenti"))
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
            .field("Status", "✅ Aktif", true)
            .field("Mention Everyone", if ch.mention_everyone { "Ya" } else { "Tidak" }, true)
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

/// Show latest equity/stock news from Core
#[poise::command(slash_command)]
pub async fn latest(
    ctx: Context<'_>,
    #[description = "Number of news items (max 10)"] limit: Option<u8>,
) -> Result<(), Error> {
    ctx.defer().await?;

    let limit = limit.unwrap_or(5).clamp(1, 10);
    let core_url = ctx.data().core_http_url.clone();
    let url = format!("{}/api/v1/equity/news?limit={}", core_url, limit);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    let response = client.get(&url).send().await;

    let items = match response {
        Ok(resp) => match resp.json::<serde_json::Value>().await {
            Ok(body) => body["items"].as_array().cloned().unwrap_or_default(),
            Err(_) => vec![],
        },
        Err(_) => vec![],
    };

    if items.is_empty() {
        let embed = CreateEmbed::new()
            .title("Equity News")
            .description("Belum ada berita equity saat ini.")
            .color(0x5865F2)
            .footer(CreateEmbedFooter::new("Fio"))
            .timestamp(Timestamp::now());
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    }

    let mut lines = Vec::new();
    for item in &items {
        let title = item["title"].as_str().unwrap_or("(no title)");
        let tickers = item["tickers"].as_str().unwrap_or("");
        let sentiment = item["sentiment"].as_str().unwrap_or("");
        let sentiment_icon = match sentiment {
            "positive" => "🟢",
            "negative" => "🔴",
            _ => "⚪",
        };
        let ticker_str = if tickers.is_empty() {
            String::new()
        } else {
            format!(" `{}`", tickers)
        };
        lines.push(format!("{} **{}**{}", sentiment_icon, title, ticker_str));
    }

    let embed = CreateEmbed::new()
        .title(format!("Latest Equity News ({} items)", items.len()))
        .description(lines.join("\n\n"))
        .color(0x5865F2)
        .footer(CreateEmbedFooter::new("Fio • Source: Core API"))
        .timestamp(Timestamp::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Search stock news by keyword
#[poise::command(slash_command)]
pub async fn search(
    ctx: Context<'_>,
    #[description = "Keyword to search"] keyword: String,
    #[description = "Number of results (max 10)"] limit: Option<i64>,
) -> Result<(), Error> {
    let embed = CreateEmbed::new()
        .title(format!("Pencarian: {}", keyword))
        .description(
            "Fitur search sekarang tersedia via dashboard web.\nBerita saham akan di-push otomatis ke channel yang subscribe.",
        )
        .color(0x2962FF)
        .footer(CreateEmbedFooter::new("Fio"));

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Get stock market summary
#[poise::command(slash_command)]
pub async fn market(ctx: Context<'_>) -> Result<(), Error> {
    let embed = CreateEmbed::new()
        .title("Ringkasan Pasar Saham Indonesia")
        .description(
            "Ringkasan pasar sekarang tersedia via dashboard web.\nBerita high impact akan otomatis di-push ke channel yang subscribe.",
        )
        .color(0x2962FF)
        .footer(CreateEmbedFooter::new("Fio"));

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
