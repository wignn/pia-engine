use super::{Context, Error, STOCK_COLOR};
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter, Timestamp};

/// Show latest equity/stock news from Core
#[poise::command(slash_command)]
pub async fn latest(
    ctx: Context<'_>,
    #[description = "Number of news items (max 10)"] limit: Option<u8>,
) -> Result<(), Error> {
    ctx.defer().await?;

    let limit = limit.unwrap_or(5).clamp(1, 10);
    let realtime_url = ctx.data().api_http_url.clone();
    let url = format!("{}/api/v1/stock/news?limit={}", realtime_url, limit);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    let response = client
        .get(&url)
        .header("X-API-Key", &ctx.data().api_key)
        .send()
        .await?;
    let body = response
        .error_for_status()?
        .json::<serde_json::Value>()
        .await?;
    let items = body["news"].as_array().cloned().unwrap_or_default();

    if items.is_empty() {
        let embed = CreateEmbed::new()
            .title("Equity News")
            .description("Belum ada berita equity saat ini.")
            .color(STOCK_COLOR)
            .footer(CreateEmbedFooter::new("Fio"))
            .timestamp(Timestamp::now());
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    }

    let mut lines = Vec::new();
    for item in &items {
        let headline = item["headline"].as_str().unwrap_or("(no headline)");
        let source = item["source"].as_str().unwrap_or("");
        let sentiment = item["sentiment"].as_str().unwrap_or("");
        let url = item["url"].as_str().unwrap_or("");
        let sentiment_icon = match sentiment {
            "bullish" => "🟢",
            "bearish" => "🔴",
            _ => "⚪",
        };
        let source_str = if source.is_empty() {
            String::new()
        } else {
            format!(" `{source}`")
        };
        let link = if url.is_empty() {
            headline.to_string()
        } else {
            format!("[{headline}]({url})")
        };
        lines.push(format!("{sentiment_icon} **{link}**{source_str}"));
    }

    let embed = CreateEmbed::new()
        .title(format!("Latest Equity News ({} items)", items.len()))
        .description(lines.join("\n\n"))
        .color(STOCK_COLOR)
        .footer(CreateEmbedFooter::new("Fio • Source: Core API"))
        .timestamp(Timestamp::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
