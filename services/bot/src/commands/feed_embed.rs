use poise::serenity_prelude::{Colour, CreateEmbed, Timestamp};

pub const DISABLED_COLOR: Colour = Colour::from_rgb(158, 158, 158);

pub fn disabled(title: &str, command: &str, label: &str) -> CreateEmbed {
    CreateEmbed::default()
        .title(title)
        .description(format!(
            "{} notifications have been disabled.\n\nUse `{}` to enable again.",
            label, command
        ))
        .color(DISABLED_COLOR)
        .timestamp(Timestamp::now())
}

pub fn enabled(title: &str, description: &str, color: Colour) -> CreateEmbed {
    CreateEmbed::default()
        .title(title)
        .description(description)
        .color(color)
        .timestamp(Timestamp::now())
}

pub fn status(
    title: &str,
    setup_command: &str,
    channel: Option<(i64, bool)>,
    active_color: Colour,
    extra_fields: &[(&str, &str, bool)],
) -> CreateEmbed {
    match channel {
        Some((channel_id, is_active)) => {
            let status = if is_active { "Active" } else { "Disabled" };
            let color = if is_active {
                active_color
            } else {
                DISABLED_COLOR
            };
            let mut embed = CreateEmbed::default()
                .title(title)
                .field("Status", status, true)
                .field("Channel", format!("<#{}>", channel_id), true)
                .color(color)
                .timestamp(Timestamp::now());

            for (name, value, inline) in extra_fields {
                embed = embed.field(*name, *value, *inline);
            }

            embed
        }
        None => CreateEmbed::default()
            .title(title)
            .description(format!(
                "Not configured. Use `{}` to enable.",
                setup_command
            ))
            .color(DISABLED_COLOR)
            .timestamp(Timestamp::now()),
    }
}
