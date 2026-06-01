use crate::commands::Data;
use crate::repository::ModerationRepository;
use crate::utils::embed;
use serenity::all::{ChannelId, Context, CreateMessage, GuildId, Member, RoleId, User};

use super::EventResult;

/// Handle new member joining the server.
pub async fn handle_member_join(ctx: &Context, new_member: &Member, data: &Data) -> EventResult {
    let guild_id = new_member.guild_id;
    let pool = data.db.as_ref();
    let config = ModerationRepository::get_config(pool, guild_id.get()).await;

    if let Ok(Some(config)) = config {
        if let Some(role_id) = config.auto_role_id {
            assign_auto_role(ctx, new_member, role_id).await;
        }

        if let Some(log_channel_id) = config.log_channel_id {
            send_join_log(ctx, new_member, log_channel_id).await;
        }
    }

    Ok(())
}

/// Handle member leaving the server.
pub async fn handle_member_leave(
    ctx: &Context,
    guild_id: GuildId,
    user: &User,
    _member_data: Option<&Member>,
    data: &Data,
) -> EventResult {
    let pool = data.db.as_ref();
    let config = ModerationRepository::get_config(pool, guild_id.get()).await;

    if let Ok(Some(config)) = config
        && let Some(log_channel_id) = config.log_channel_id
    {
        send_leave_log(ctx, guild_id, user, log_channel_id).await;
    }

    Ok(())
}

async fn assign_auto_role(ctx: &Context, new_member: &Member, role_id: i64) {
    let role = RoleId::new(role_id as u64);
    let member = new_member.clone();
    if let Err(e) = member.add_role(&ctx.http, role).await {
        eprintln!("[MOD] Failed to assign auto-role: {}", e);
    }
}

async fn send_join_log(ctx: &Context, new_member: &Member, log_channel_id: i64) {
    let channel = ChannelId::new(log_channel_id as u64);
    let member_count = ctx
        .cache
        .guild(new_member.guild_id)
        .map(|g| g.member_count)
        .unwrap_or(0);
    let account_created = new_member
        .user
        .created_at()
        .format("%Y-%m-%d %H:%M UTC")
        .to_string();
    let avatar = new_member.user.avatar_url();

    let embed_msg = embed::member_join(
        &new_member.user.name,
        new_member.user.id.get(),
        member_count,
        avatar.as_deref(),
        &account_created,
    );
    let message = CreateMessage::new().embed(embed_msg);

    if let Err(e) = channel.send_message(&ctx.http, message).await {
        eprintln!("[MOD] Failed to send join log: {}", e);
    }
}

async fn send_leave_log(ctx: &Context, guild_id: GuildId, user: &User, log_channel_id: i64) {
    let channel = ChannelId::new(log_channel_id as u64);
    let guild_name = ctx
        .cache
        .guild(guild_id)
        .map(|g| g.name.clone())
        .unwrap_or_else(|| "Server".to_string());
    let avatar = user.avatar_url();

    let embed_msg = embed::member_leave(&user.name, user.id.get(), avatar.as_deref(), &guild_name);
    let message = CreateMessage::new().embed(embed_msg);

    if let Err(e) = channel.send_message(&ctx.http, message).await {
        eprintln!("[MOD] Failed to send leave log: {}", e);
    }
}
