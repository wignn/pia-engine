use atlsd_common::dlq::DeadLetterQueue;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{debug, info, warn};

use crate::collector::stock::StockCollector;
use crate::ws::{self, Hub, StockNewsData};

const MAX_STOCK_NEWS_AGE_HOURS: i64 = 12;

pub struct StockPipeline {
    collector: Arc<StockCollector>,
    db: PgPool,
    hub: Arc<Hub>,
    dlq: Option<DeadLetterQueue>,
}

impl StockPipeline {
    pub fn new(
        collector: Arc<StockCollector>,
        db: PgPool,
        hub: Arc<Hub>,
        redis_client: Option<redis::Client>,
        redis_channel_prefix: &str,
    ) -> Self {
        let dlq = redis_client.map(|client| {
            DeadLetterQueue::new(
                client,
                format!("{}:dlq:stock-news", redis_channel_prefix),
                10_000,
            )
        });

        Self {
            collector,
            db,
            hub,
            dlq,
        }
    }

    pub async fn run(&self) {
        info!("stock pipeline: starting");
        let entries = self.collector.fetch_latest(30).await;
        info!(count = entries.len(), "stock pipeline: entries fetched");

        let mut processed = 0u32;
        let mut too_old = 0u32;
        let mut duplicate = 0u32;
        let mut db_error = 0u32;
        let mut skipped = 0u32;

        for entry in &entries {
            match self.process_entry(entry).await {
                "processed" => processed += 1,
                "too_old" => too_old += 1,
                "duplicate" => duplicate += 1,
                "db_error" => db_error += 1,
                _ => skipped += 1,
            }
        }

        info!(
            processed,
            too_old, duplicate, db_error, skipped, "stock pipeline: completed"
        );
    }

    async fn process_entry(&self, entry: &crate::collector::stock::StockNewsEntry) -> &'static str {
        if let Some(pub_at) = entry.published_at {
            if is_too_old(pub_at, Utc::now()) {
                debug!(
                    title = %truncate_title(&entry.title, 60),
                    age_hours = (Utc::now() - pub_at).num_hours(),
                    "stock article skipped: too_old"
                );
                return "too_old";
            }
        }

        let is_duplicate = match sqlx::query_as::<_, (bool,)>(
            "SELECT EXISTS(SELECT 1 FROM news.stock_news WHERE content_hash = $1)",
        )
        .bind(&entry.content_hash)
        .fetch_one(&self.db)
        .await
        {
            Ok((true,)) => true,
            Ok((false,)) => false,
            Err(e) => {
                warn!(error = %e, hash = %entry.content_hash, "stock duplicate check failed, continuing");
                false
            }
        };

        if is_duplicate {
            debug!(hash = %entry.content_hash, title = %truncate_title(&entry.title, 60), "stock article skipped: duplicate");
            return "duplicate";
        }

        let impact_level = impact_level(entry.tickers.len());

        let published_at = entry
            .published_at
            .map(|d| d.to_rfc3339())
            .unwrap_or_default();
        let tickers_str = entry.tickers.join(",");

        let sentiment_val =
            crate::pipeline::sentiment::analyze(&format!("{} - {}", entry.title, entry.content))
                .await;

        let res = sqlx::query(
            "INSERT INTO news.stock_news \
             (content_hash, original_url, title, source_name, category, \
              tickers, sentiment, impact_level, is_processed, processed_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, TRUE, NOW()) \
             ON CONFLICT (content_hash) DO NOTHING",
        )
        .bind(&entry.content_hash)
        .bind(&entry.link)
        .bind(&entry.title)
        .bind(&entry.source_name)
        .bind(&entry.category)
        .bind(&tickers_str)
        .bind(&sentiment_val)
        .bind(impact_level)
        .execute(&self.db)
        .await;

        match res {
            Ok(r) => {
                if r.rows_affected() == 0 {
                    debug!(hash = %entry.content_hash, title = %truncate_title(&entry.title, 60), "stock article skipped: duplicate (ON CONFLICT)");
                    return "duplicate";
                }
                info!(title = %truncate_title(&entry.title, 50), source = %entry.source_name, "stock article saved");
            }
            Err(e) => {
                warn!(error = %e, "stock db insert failed");
                if let Some(dlq) = &self.dlq {
                    let payload = serde_json::json!({
                        "content_hash": entry.content_hash,
                        "link": entry.link,
                        "title": entry.title,
                        "content": entry.content,
                        "source_name": entry.source_name,
                        "category": entry.category,
                        "tickers": entry.tickers,
                        "published_at": entry.published_at.map(|d| d.to_rfc3339()),
                    });
                    dlq.push("stock-news", &entry.content_hash, &e.to_string(), &payload)
                        .await;
                }
                return "db_error";
            }
        }

        let summary_truncated = truncate_summary(&entry.content);

        let stock_data = StockNewsData {
            id: entry.content_hash.clone(),
            title: entry.title.clone(),
            summary: Some(summary_truncated),
            content: None,
            source_name: entry.source_name.clone(),
            source_url: entry.link.clone(),
            url: entry.link.clone(),
            category: entry.category.clone(),
            tickers: entry.tickers.clone(),
            sentiment: Some(sentiment_val),
            impact_level: Some(impact_level.to_string()),
            published_at: if published_at.is_empty() {
                None
            } else {
                Some(published_at)
            },
            processed_at: Utc::now().to_rfc3339(),
        };

        let embed = ws::build_stock_embed(&stock_data);
        let data = serde_json::json!({
            "article": stock_data,
            "discord_embed": embed,
            "asset_type": "stock",
        });
        let count = self
            .hub
            .broadcast(ws::EVENT_STOCK_NEWS_NEW, data, "stock_news")
            .await;

        info!(clients = count, title = %truncate_title(&entry.title, 50), tickers = ?entry.tickers, "stock broadcast ok");

        "processed"
    }
}

fn is_too_old(published_at: DateTime<Utc>, now: DateTime<Utc>) -> bool {
    published_at < now - chrono::Duration::hours(MAX_STOCK_NEWS_AGE_HOURS)
}

fn impact_level(ticker_count: usize) -> &'static str {
    if ticker_count >= 3 {
        "high"
    } else if ticker_count > 0 {
        "medium"
    } else {
        "low"
    }
}

fn truncate_summary(content: &str) -> String {
    if content.len() > 1000 {
        content[..1000].to_string()
    } else {
        content.to_string()
    }
}

fn truncate_title(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        s[..max_len].to_string()
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_impact_level_from_ticker_count() {
        assert_eq!(impact_level(0), "low");
        assert_eq!(impact_level(1), "medium");
        assert_eq!(impact_level(2), "medium");
        assert_eq!(impact_level(3), "high");
    }

    #[test]
    fn detects_articles_older_than_cutoff() {
        let now = Utc::now();

        assert!(!is_too_old(now - chrono::Duration::hours(12), now));
        assert!(is_too_old(now - chrono::Duration::hours(13), now));
    }

    #[test]
    fn truncates_summary_to_limit() {
        let long = "a".repeat(1001);
        let truncated = truncate_summary(&long);

        assert_eq!(truncate_summary("short"), "short");
        assert_eq!(truncated.len(), 1000);
    }

    #[test]
    fn truncates_title_to_limit() {
        assert_eq!(truncate_title("abcdef", 3), "abc");
        assert_eq!(truncate_title("abc", 3), "abc");
    }
}
