use crate::commands::{Context, Error};
use poise::serenity_prelude as serenity;
use serenity::{CreateEmbed, CreateEmbedFooter, Timestamp};

/// Get current high impact forex events
#[poise::command(slash_command, prefix_command, aliases("calendar"))]
pub async fn forex_calendar(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let realtime_url = ctx.data().api_http_url.clone();
    let url = format!(
        "{}/api/v1/forex/calendar?impact=high&limit=10",
        realtime_url
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;
    let response = client.get(&url).send().await;

    let mut high_impact_events = Vec::new();
    if let Ok(resp) = response
        && let Ok(body) = resp.json::<serde_json::Value>().await
        && let Some(items) = body["items"].as_array()
    {
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
                if forecast.is_empty() { "-" } else { forecast },
                if previous.is_empty() { "-" } else { previous }
            ));
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
