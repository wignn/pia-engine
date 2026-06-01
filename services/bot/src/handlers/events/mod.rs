mod member;
mod voice;

use crate::commands::Data;
use serenity::all::{Context, FullEvent};

pub type EventResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

/// Main event handler for Discord events.
pub async fn handle_event(ctx: &Context, event: &FullEvent, data: &Data) -> EventResult {
    match event {
        FullEvent::VoiceStateUpdate { old, new } => {
            voice::handle_voice_state_update(ctx, old, new, data).await?;
        }
        FullEvent::GuildMemberAddition { new_member } => {
            member::handle_member_join(ctx, new_member, data).await?;
        }
        FullEvent::GuildMemberRemoval {
            guild_id,
            user,
            member_data_if_available,
        } => {
            member::handle_member_leave(
                ctx,
                *guild_id,
                user,
                member_data_if_available.as_ref(),
                data,
            )
            .await?;
        }
        _ => {}
    }

    Ok(())
}
