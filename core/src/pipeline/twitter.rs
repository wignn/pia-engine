use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};

use crate::collector::twitter::TwitterCollector;
use crate::ws::{self, Hub, XPostData};

pub struct TwitterPipeline {
    collector: Arc<TwitterCollector>,
    hub: Arc<Hub>,
    env_usernames: String,
}

impl TwitterPipeline {
    pub fn new(
        collector: Arc<TwitterCollector>,
        hub: Arc<Hub>,
        env_usernames: String,
        _tenant_registry: Option<Arc<crate::tenant::registry::TenantRegistry>>,
    ) -> Self {
        Self { collector, hub, env_usernames }
    }

    pub async fn run(&self, interval: Duration) {
        info!(interval = ?interval, usernames = %self.env_usernames, "twitter pipeline: starting (env-only)");

        self.seed(&self.env_usernames).await;

        let mut ticker = tokio::time::interval(interval);
        ticker.tick().await;

        loop {
            ticker.tick().await;
            self.tick(&self.env_usernames).await;
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
