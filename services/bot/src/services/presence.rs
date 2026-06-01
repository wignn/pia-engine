use serenity::all::{ActivityData, OnlineStatus, ShardManager};
use std::sync::Arc;

pub fn spawn_presence_loop(shard_manager: Arc<ShardManager>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        let mut idx = 0usize;

        loop {
            interval.tick().await;

            let mut activities = Vec::new();
            if let Some(xau) = crate::services::market_ws::get_xauusd_display() {
                activities.push(ActivityData::custom(xau));
            }

            if activities.is_empty() {
                continue;
            }

            let runners = shard_manager.runners.lock().await;
            for (_, runner) in runners.iter() {
                runner.runner_tx.set_presence(
                    Some(activities[idx % activities.len()].clone()),
                    OnlineStatus::Online,
                );
            }
            idx = (idx + 1) % activities.len();
        }
    });
}
