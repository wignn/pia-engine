use crate::repository::ForexRepository;
use poise::serenity_prelude as serenity;
use serenity::{CreateEmbed, CreateEmbedFooter, Timestamp};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, super::Data, Error>;

#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "ADMINISTRATOR"
)]
pub async fn forex_news_setup(
    ctx: Context<'_>,
    #[description = "Channel for forex news"] channel: serenity::GuildChannel,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Must be used in a guild")?.get();
    let channel_id = channel.id.get();

    let pool = ctx.data().db.as_ref();
    ForexRepository::insert_channel(pool, guild_id, channel_id).await?;

    let embed = CreateEmbed::default()
        .title("Forex News Setup Complete")
        .description(format!(
            "Real-time forex news will be sent to <#{}>\n\n\
            **Coverage:**\n\
            `USD` `EUR` `GBP` `JPY` `CHF` `AUD` `NZD` `CAD`\n\n\
            **Sources:**\n\
            FXStreet, Forex Factory, Investing.com\n\n\
            **Impact Levels:**\n\
            `HIGH` - Central bank decisions, NFP, CPI, GDP\n\
            `MEDIUM` - Trade balance, PMI, Housing data",
            channel_id
        ))
        .color(serenity::Colour::from_rgb(0, 150, 136))
        .footer(CreateEmbedFooter::new("Updates every 60 seconds"))
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
pub async fn forex_news_disable(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Must be used in a guild")?.get();

    let pool = ctx.data().db.as_ref();
    ForexRepository::disable_channel(pool, guild_id).await?;

    let embed = CreateEmbed::default()
        .title("Forex News Disabled")
        .description(
            "Forex news notifications have been disabled.\n\nUse `/forex_news_setup` to enable again.",
        )
        .color(serenity::Colour::from_rgb(158, 158, 158))
        .timestamp(Timestamp::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Enable forex news notifications
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "ADMINISTRATOR"
)]
pub async fn forex_news_enable(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Must be used in a guild")?.get();

    let pool = ctx.data().db.as_ref();
    ForexRepository::enable_channel(pool, guild_id).await?;

    let embed = CreateEmbed::default()
        .title("Forex News Enabled")
        .description("Forex news notifications have been re-enabled.")
        .color(serenity::Colour::from_rgb(0, 150, 136))
        .timestamp(Timestamp::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Check forex news status
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn forex_news_status(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Must be used in a guild")?.get();

    let pool = ctx.data().db.as_ref();
    let channel = ForexRepository::get_channel(pool, guild_id).await?;

    let embed = match channel {
        Some(ch) => {
            let status = if ch.is_active { "Active" } else { "Disabled" };
            let color = if ch.is_active {
                serenity::Colour::from_rgb(0, 150, 136)
            } else {
                serenity::Colour::from_rgb(158, 158, 158)
            };

            CreateEmbed::default()
                .title("Forex News Status")
                .field("Status", status, true)
                .field("Channel", format!("<#{}>", ch.channel_id), true)
                .color(color)
                .timestamp(Timestamp::now())
        }
        None => CreateEmbed::default()
            .title("Forex News Status")
            .description("Not configured. Use `/forex_news_setup` to enable.")
            .color(serenity::Colour::from_rgb(158, 158, 158))
            .timestamp(Timestamp::now()),
    };

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Get current high impact forex events
#[poise::command(slash_command, prefix_command, aliases("calendar"))]
pub async fn forex_calendar(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let core_url = ctx.data().core_http_url.clone();
    let url = format!("{}/api/v1/forex/calendar?impact=high&limit=10", core_url);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    let response = client.get(&url).send().await;

    let mut high_impact_events: Vec<String> = Vec::new();

    if let Ok(resp) = response {
        if let Ok(body) = resp.json::<serde_json::Value>().await {
            if let Some(items) = body["items"].as_array() {
                for event in items {
                    let title = event["title"].as_str().unwrap_or_default();
                    let currency = event["currency"].as_str().unwrap_or_default();
                    let date = event["date"].as_str().unwrap_or_default();
                    let forecast = event["forecast"].as_str().unwrap_or_default();
                    let previous = event["previous"].as_str().unwrap_or_default();

                    high_impact_events.push(format!(
                        "**{}**  `{}`\n{}\nForecast: `{}` | Previous: `{}`",
                        currency,
                        date,
                        title,
                        if forecast.is_empty() { "—" } else { forecast },
                        if previous.is_empty() { "—" } else { previous }
                    ));
                }
            }
        }
    }

    let description = if high_impact_events.is_empty() {
        "No high impact events scheduled.\n\nCheck back later or visit [Forex Factory](https://www.forexfactory.com/calendar) for the full calendar.".to_string()
    } else {
        high_impact_events.join("\n\n")
    };

    let embed = CreateEmbed::default()
        .title("HIGH IMPACT FOREX CALENDAR")
        .description(description)
        .color(serenity::Colour::from_rgb(220, 53, 69))
        .footer(CreateEmbedFooter::new("Source: Forex Factory via Core"))
        .timestamp(Timestamp::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
