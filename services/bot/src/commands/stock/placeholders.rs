use super::{Context, Error, STOCK_ACTION_COLOR};
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter};

/// Search stock news by keyword
#[poise::command(slash_command)]
pub async fn search(
    ctx: Context<'_>,
    #[description = "Keyword to search"] keyword: String,
    #[description = "Number of results (max 10)"] _limit: Option<i64>,
) -> Result<(), Error> {
    let embed = CreateEmbed::new()
        .title(format!("Pencarian: {}", keyword))
        .description(
            "Fitur search sekarang tersedia via dashboard web.\nBerita saham akan di-push otomatis ke channel yang subscribe.",
        )
        .color(STOCK_ACTION_COLOR)
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
        .color(STOCK_ACTION_COLOR)
        .footer(CreateEmbedFooter::new("Fio"));

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
