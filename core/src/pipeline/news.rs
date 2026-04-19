use std::sync::Arc;

use chrono::Utc;
use redis::streams::StreamReadReply;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::sync::LazyLock;
use tracing::{debug, error, info, warn};

use crate::collector::rss::{default_forex_feeds, feed_name_by_url, RSSCollector};
use crate::html;
use crate::scraper::article::ArticleScraper;
use crate::ws::{self, Hub, NewsArticleData};

const MAX_NEWS_AGE_HOURS: i64 = 2;
const REDIS_STREAM_MAX_LEN: usize = 50_000;

static SLUG_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[^a-z0-9]+").unwrap());

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NewsIngestMessage {
    source_name: String,
    feed_url: String,
    title: String,
    link: String,
    content: String,
    published_at: Option<String>,
    content_hash: String,
}

pub struct NewsPipeline {
    rss: Arc<RSSCollector>,
    scraper: Arc<ArticleScraper>,
    db: PgPool,
    hub: Arc<Hub>,
    redis_client: Option<redis::Client>,
    stream_key: String,
}

pub struct NewsIngestWorker {
    scraper: Arc<ArticleScraper>,
    db: PgPool,
    hub: Arc<Hub>,
    redis_client: redis::Client,
    stream_key: String,
}

impl NewsPipeline {
    pub fn new(
        rss: Arc<RSSCollector>,
        scraper: Arc<ArticleScraper>,
        db: PgPool,
        hub: Arc<Hub>,
        redis_client: Option<redis::Client>,
        redis_channel_prefix: &str,
    ) -> Self {
        Self {
            rss,
            scraper,
            db,
            hub,
            redis_client,
            stream_key: format!("{}:stream:news:ingest", redis_channel_prefix),
        }
    }

    pub async fn run(&self) {
        info!(stream_key = %self.stream_key, redis_enabled = self.redis_client.is_some(), "news ingest producer: starting");
        let feeds = default_forex_feeds();
        let results = self.rss.fetch_all_feeds(&feeds).await;

        let total: usize = results.values().map(|v| v.len()).sum();
        info!(feeds = results.len(), total_entries = total, "news ingest producer: feeds fetched");

        let mut enqueued = 0u32;
        let mut direct_processed = 0u32;
        let mut skipped = 0u32;

        for (feed_url, entries) in &results {
            let source_name = feed_name_by_url(feed_url);
            for entry in entries {
                let msg = NewsIngestMessage {
                    source_name: source_name.clone(),
                    feed_url: feed_url.clone(),
                    title: entry.title.clone(),
                    link: entry.link.clone(),
                    content: entry.content.clone(),
                    published_at: entry.published_at.map(|d| d.to_rfc3339()),
                    content_hash: entry.content_hash.clone(),
                };

                if self.enqueue_message(&msg).await {
                    enqueued += 1;
                    continue;
                }

                // Fallback mode: if Redis is unavailable, process directly.
                match process_entry(
                    &self.scraper,
                    &self.db,
                    &self.hub,
                    &msg,
                )
                .await
                {
                    "processed" => direct_processed += 1,
                    _ => skipped += 1,
                }
            }
        }

        info!(enqueued, direct_processed, skipped, "news ingest producer: completed");
    }

    pub fn build_worker(&self) -> Option<NewsIngestWorker> {
        let redis_client = self.redis_client.clone()?;
        Some(NewsIngestWorker {
            scraper: self.scraper.clone(),
            db: self.db.clone(),
            hub: self.hub.clone(),
            redis_client,
            stream_key: self.stream_key.clone(),
        })
    }

    async fn enqueue_message(&self, msg: &NewsIngestMessage) -> bool {
        let Some(redis_client) = &self.redis_client else {
            return false;
        };

        let payload = match serde_json::to_string(msg) {
            Ok(p) => p,
            Err(e) => {
                warn!(error = %e, "failed to serialize news ingest message");
                return false;
            }
        };

        let mut conn = match redis_client.get_multiplexed_async_connection().await {
            Ok(c) => c,
            Err(e) => {
                warn!(error = %e, "redis connection failed for news ingest enqueue");
                return false;
            }
        };

        let enqueue_result: redis::RedisResult<String> = redis::cmd("XADD")
            .arg(&self.stream_key)
            .arg("MAXLEN")
            .arg("~")
            .arg(REDIS_STREAM_MAX_LEN)
            .arg("*")
            .arg("payload")
            .arg(payload)
            .query_async(&mut conn)
            .await;

        match enqueue_result {
            Ok(_) => true,
            Err(e) => {
                warn!(error = %e, stream = %self.stream_key, "redis XADD failed for news ingest");
                false
            }
        }
    }
}

impl NewsIngestWorker {
    pub async fn run_forever(&self) {
        let mut last_id = "$".to_string();
        info!(stream_key = %self.stream_key, "news ingest worker started");

        loop {
            let mut conn = match self.redis_client.get_multiplexed_async_connection().await {
                Ok(c) => c,
                Err(e) => {
                    error!(error = %e, "news ingest worker redis connection failed");
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    continue;
                }
            };

            let reply: redis::RedisResult<StreamReadReply> = redis::cmd("XREAD")
                .arg("COUNT")
                .arg(50)
                .arg("BLOCK")
                .arg(5000)
                .arg("STREAMS")
                .arg(&self.stream_key)
                .arg(&last_id)
                .query_async(&mut conn)
                .await;

            let reply = match reply {
                Ok(r) => r,
                Err(e) => {
                    warn!(error = %e, stream = %self.stream_key, "news ingest worker XREAD failed");
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    continue;
                }
            };

            for key in reply.keys {
                for id in key.ids {
                    let msg_id = id.id.clone();
                    let payload = id
                        .map
                        .get("payload")
                        .and_then(|v| redis::from_redis_value::<String>(v).ok());

                    let Some(payload) = payload else {
                        warn!(msg_id = %msg_id, "news ingest message missing payload field");
                        last_id = msg_id;
                        continue;
                    };

                    let msg: NewsIngestMessage = match serde_json::from_str(&payload) {
                        Ok(m) => m,
                        Err(e) => {
                            warn!(msg_id = %msg_id, error = %e, "failed to deserialize news ingest payload");
                            last_id = msg_id;
                            continue;
                        }
                    };

                    let result = process_entry(&self.scraper, &self.db, &self.hub, &msg).await;
                    if result == "processed" {
                        info!(msg_id = %msg_id, title = %truncate_str(&msg.title, 50), "news ingest processed");
                    }

                    last_id = msg_id;
                }
            }
        }
    }
}

async fn process_entry(
    scraper: &Arc<ArticleScraper>,
    db: &PgPool,
    hub: &Arc<Hub>,
    msg: &NewsIngestMessage,
) -> &'static str {
    if let Some(pub_at) = parse_rfc3339_utc(msg.published_at.as_deref()) {
        let cutoff = Utc::now() - chrono::Duration::hours(MAX_NEWS_AGE_HOURS);
        if pub_at < cutoff {
            return "too_old";
        }
    }

    let hash = &msg.content_hash;
    let exists: Option<(bool,)> = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM news_articles WHERE content_hash = $1)")
        .bind(hash)
        .fetch_optional(db)
        .await
        .ok()
        .flatten();

    if let Some((true,)) = exists {
        return "duplicate";
    }

    let title = &msg.title;
    let mut content = html::strip_tags(&msg.content);
    let description = content.clone();
    let mut image_url = String::new();
    let mut published_at = msg.published_at.clone().unwrap_or_default();

    if content.len() < 200 && !msg.link.is_empty() {
        match scraper.scrape(&msg.link).await {
            Ok(article) => {
                content = article.content;
                if !article.image_url.is_empty() {
                    image_url = article.image_url;
                }
                if !article.published_at.is_empty() && published_at.is_empty() {
                    published_at = article.published_at;
                }
            }
            Err(e) => debug!(url = %msg.link, error = %e, "scrape failed, using rss fallback"),
        }
    }

    let summary = {
        let s = html::extract_summary(&description, 500);
        if s.is_empty() {
            html::extract_summary(&content, 500)
        } else {
            s
        }
    };

    let source_id = ensure_source(db, &msg.source_name, &msg.feed_url).await;
    let content_truncated = truncate_str(&content, 5000);
    let pub_at_opt: Option<&str> = if published_at.is_empty() {
        None
    } else {
        Some(&published_at)
    };

    let res = sqlx::query(
        "INSERT INTO news_articles \
         (id, source_id, content_hash, original_url, original_title, original_content, \
          translated_title, summary, is_processed, processed_at, published_at) \
         VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, '', $6, TRUE, NOW(), $7::timestamptz) \
         ON CONFLICT (content_hash) DO NOTHING",
    )
    .bind(&source_id)
    .bind(hash)
    .bind(&msg.link)
    .bind(title)
    .bind(&content_truncated)
    .bind(&summary)
    .bind(pub_at_opt)
    .execute(db)
    .await;

    match res {
        Ok(_) => info!(title = %truncate_str(title, 50), source = %msg.source_name, "article saved"),
        Err(e) => {
            warn!(error = %e, url = %msg.link, "db insert failed");
            return "db_error";
        }
    }

    let article_data = NewsArticleData {
        id: hash.clone(),
        title: title.clone(),
        title_id: None,
        source_name: msg.source_name.clone(),
        source_url: msg.feed_url.clone(),
        url: msg.link.clone(),
        summary: Some(summary),
        summary_id: None,
        sentiment: None,
        impact_level: Some("medium".to_string()),
        published_at: if published_at.is_empty() { None } else { Some(published_at) },
        processed_at: Utc::now().to_rfc3339(),
        image_url: if image_url.is_empty() { None } else { Some(image_url) },
    };

    let embed = ws::build_news_embed(&article_data);
    let data = serde_json::json!({
        "article": article_data,
        "discord_embed": embed,
    });
    let count = hub.broadcast(ws::EVENT_NEWS_NEW, data, "news").await;
    info!(clients = count, title = %truncate_str(title, 50), "broadcast ok");

    "processed"
}

async fn ensure_source(db: &PgPool, source_name: &str, feed_url: &str) -> String {
    let slug = to_slug(source_name);
    let slug = if slug.is_empty() {
        "unknown".to_string()
    } else {
        slug
    };

    let existing: Option<(String,)> = sqlx::query_as("SELECT id FROM news_sources WHERE slug = $1 LIMIT 1")
        .bind(&slug)
        .fetch_optional(db)
        .await
        .ok()
        .flatten();

    if let Some((id,)) = existing {
        return id;
    }

    let hash = format!(
        "{:x}",
        Sha256::new()
            .chain_update(format!("source-{}", slug))
            .finalize()
    );
    let new_id = format!(
        "{}-{}-{}-{}-{}",
        &hash[0..8],
        &hash[8..12],
        &hash[12..16],
        &hash[16..20],
        &hash[20..32]
    );

    let display_name = if source_name.is_empty() {
        "Unknown"
    } else {
        source_name
    };
    let source_url = if feed_url.is_empty() {
        "https://unknown.com"
    } else {
        feed_url
    };

    let _ = sqlx::query(
        "INSERT INTO news_sources (id, name, slug, source_type, url, rss_url, is_active) \
         VALUES ($1, $2, $3, 'rss', $4, $5, TRUE) \
         ON CONFLICT (slug) DO NOTHING",
    )
    .bind(&new_id)
    .bind(display_name)
    .bind(&slug)
    .bind(source_url)
    .bind(feed_url)
    .execute(db)
    .await;

    let re_read: Option<(String,)> = sqlx::query_as("SELECT id FROM news_sources WHERE slug = $1 LIMIT 1")
        .bind(&slug)
        .fetch_optional(db)
        .await
        .ok()
        .flatten();

    re_read.map(|(id,)| id).unwrap_or(new_id)
}

fn parse_rfc3339_utc(input: Option<&str>) -> Option<chrono::DateTime<Utc>> {
    input
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&Utc))
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        s[..max_len].to_string()
    } else {
        s.to_string()
    }
}

fn to_slug(name: &str) -> String {
    let s = name.trim().to_lowercase();
    let s = SLUG_RE.replace_all(&s, "-");
    s.trim_matches('-').to_string()
}
