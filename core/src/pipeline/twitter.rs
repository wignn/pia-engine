use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};

use crate::collector::twitter::TwitterCollector;
use crate::tenant::registry::TenantRegistry;
use crate::ws::{self, Hub, XPostData};

pub struct TwitterPipeline {
    collector: Arc<TwitterCollector>,
    hub: Arc<Hub>,
    env_usernames: String,
    tenant_registry: Option<Arc<TenantRegistry>>,
}

impl TwitterPipeline {
    pub fn new(
        collector: Arc<TwitterCollector>,
        hub: Arc<Hub>,
        env_usernames: String,
        tenant_registry: Option<Arc<TenantRegistry>>,
    ) -> Self {
        Self { collector, hub, env_usernames, tenant_registry }
    }

    /// Merge env usernames + all tenant usernames into a single comma-separated string.
    async fn merged_usernames(&self) -> String {
        let mut all = std::collections::HashSet::new();

        // Add env usernames
        for u in self.env_usernames.split(',').map(|s| s.trim().to_lowercase()) {
            if !u.is_empty() { all.insert(u); }
        }

        // Add tenant usernames
        if let Some(registry) = &self.tenant_registry {
            let tenant_users = registry.all_x_usernames(&self.env_usernames).await;
            all.extend(tenant_users);
        }

        all.into_iter().collect::<Vec<_>>().join(",")
    }

    pub async fn run(&self, interval: Duration) {
        let merged = self.merged_usernames().await;
        info!(interval = ?interval, usernames = %merged, "twitter pipeline: starting (tenant-merged)");

        // Seed: fetch once without broadcasting to establish last-seen IDs
        self.seed(&merged).await;

        let mut ticker = tokio::time::interval(interval);
        ticker.tick().await;

        loop {
            ticker.tick().await;
            // Re-merge on every tick to pick up config changes
            let merged = self.merged_usernames().await;
            self.tick(&merged).await;
        }
    }

    async fn seed(&self, usernames: &str) {
        let tweets = self.collector.fetch_tweets(usernames).await;
        info!(tweets_seen = tweets.len(), "twitter pipeline: seeded (skipped broadcast)");
    }

    async fn tick(&self, usernames: &str) {
        let tweets = self.collector.fetch_tweets(usernames).await;
        if tweets.is_empty() { return; }

        let mut broadcasted = 0u32;
        for tweet in &tweets {
            let post_data = XPostData {
                id: tweet.id.clone(),
                text: tweet.text.clone(),
                author_username: tweet.author_username.clone(),
                author_name: tweet.author_name.clone(),
                author_avatar: if tweet.author_avatar.is_empty() { None } else { Some(tweet.author_avatar.clone()) },
                created_at: if tweet.created_at.is_empty() { None } else { Some(tweet.created_at.clone()) },
                url: tweet.url.clone(),
                media_urls: if tweet.media_urls.is_empty() { None } else { Some(tweet.media_urls.clone()) },
            };

            let embed = ws::build_x_embed(&post_data);
            let data = serde_json::json!({
                "post": post_data,
                "discord_embed": embed,
            });
            let count = self.hub.broadcast(ws::EVENT_X_NEW, data, "x").await;

            if count > 0 { broadcasted += 1; }
            info!(author = %tweet.author_username, tweet_id = %tweet.id, clients = count, "twitter: broadcast tweet");
        }

        info!(total = tweets.len(), broadcasted, "twitter pipeline: tick completed");
    }
}
