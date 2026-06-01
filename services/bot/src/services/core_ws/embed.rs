use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter};

use super::types::DiscordEmbed;

/// Converts the core service's Discord embed payload into a Serenity embed.
pub fn build_embed(de: &DiscordEmbed) -> CreateEmbed {
    let mut embed = CreateEmbed::new();
    if let Some(t) = &de.title {
        embed = embed.title(t);
    }
    if let Some(d) = &de.description {
        embed = embed.description(d);
    }
    if let Some(u) = &de.url {
        embed = embed.url(u);
    }
    if let Some(c) = de.color {
        embed = embed.color(c);
    }
    if let Some(fields) = &de.fields {
        for f in fields {
            embed = embed.field(&f.name, &f.value, f.inline);
        }
    }
    if let Some(th) = &de.thumbnail {
        embed = embed.thumbnail(&th.url);
    }
    if let Some(img) = &de.image {
        embed = embed.image(&img.url);
    }
    if let Some(ft) = &de.footer {
        embed = embed.footer(CreateEmbedFooter::new(&ft.text));
    }
    embed
}
