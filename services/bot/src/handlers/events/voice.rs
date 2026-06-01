use crate::commands::Data;
use crate::repository::ModerationRepository;
use crate::utils::embed;
use serenity::all::{ChannelId, Context, CreateMessage, GuildId, VoiceState};

use super::EventResult;

/// Handle voice state updates (join/leave voice channels).
pub async fn handle_voice_state_update(
    ctx: &Context,
    old: &Option<VoiceState>,
    new: &VoiceState,
    data: &Data,
) -> EventResult {
    if let Ok(user) = ctx.http.get_user(new.user_id).await
        && user.bot
    {
        return Ok(());
    }

    let old_channel = old.as_ref().and_then(|vs| vs.channel_id);
    let new_channel = new.channel_id;

    if let Some(guild_id) = new.guild_id {
        handle_voice_logging(ctx, data, guild_id, old_channel, new_channel, new.user_id).await?;
    }

    Ok(())
}

async fn handle_voice_logging(
    ctx: &Context,
    data: &Data,
    guild_id: GuildId,
    old_channel: Option<ChannelId>,
    new_channel: Option<ChannelId>,
    user_id: serenity::all::UserId,
) -> EventResult {
    let pool = data.db.as_ref();
    let config = ModerationRepository::get_config(pool, guild_id.get()).await;

    if let Ok(Some(config)) = config
        && let Some(log_channel_id) = config.log_channel_id
    {
        let log_channel = ChannelId::new(log_channel_id as u64);
        let user = ctx.http.get_user(user_id).await?;
        let avatar = user.avatar_url();

        if new_channel.is_some()
            && old_channel != new_channel
            && let Some(joined_channel_id) = new_channel
        {
            let channel_name = get_channel_name(ctx, guild_id, joined_channel_id);
            let embed_msg =
                embed::voice_join(&user.name, user.id.get(), &channel_name, avatar.as_deref());
            let message = CreateMessage::new().embed(embed_msg);
            let _ = log_channel.send_message(&ctx.http, message).await;
        }

        if old_channel.is_some()
            && old_channel != new_channel
            && let Some(left_channel_id) = old_channel
        {
            let channel_name = get_channel_name(ctx, guild_id, left_channel_id);
            let embed_msg =
                embed::voice_leave(&user.name, user.id.get(), &channel_name, avatar.as_deref());
            let message = CreateMessage::new().embed(embed_msg);
            let _ = log_channel.send_message(&ctx.http, message).await;
        }
    }

    Ok(())
}

fn get_channel_name(ctx: &Context, guild_id: GuildId, channel_id: ChannelId) -> String {
    ctx.cache
        .guild(guild_id)
        .and_then(|g| g.channels.get(&channel_id).map(|c| c.name.clone()))
        .unwrap_or_else(|| "Unknown".to_string())
}
