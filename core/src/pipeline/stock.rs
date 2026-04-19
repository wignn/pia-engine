use std::sync::Arc;
use chrono::Utc;
use sqlx::PgPool;
use tracing::{info, warn};

use crate::collector::stock::StockCollector;
use crate::ws::{self, EquityNewsData, Hub};

const MAX_STOCK_NEWS_AGE_HOURS: i64 = 2;

pub struct StockPipeline {
    collector: Arc<StockCollector>,
    db: PgPool,
    hub: Arc<Hub>,
}

impl StockPipeline {
    pub fn new(collector: Arc<StockCollector>, db: PgPool, hub: Arc<Hub>) -> Self {
        Self { collector, db, hub }
    }

    pub async fn run(&self) {
        info!("stock pipeline: starting");
        let entries = self.collector.fetch_latest(30).await;
        info!(count = entries.len(), "stock pipeline: entries fetched");

        let mut processed = 0u32;
        let mut skipped = 0u32;

        for entry in &entries {
            match self.process_entry(entry).await {
                "processed" => processed += 1,
                _ => skipped += 1,
            }
        }

        info!(processed, skipped, "stock pipeline: completed");
    }

    async fn process_entry(&self, entry: &crate::collector::stock::StockNewsEntry) -> &'static str {
        if let Some(pub_at) = entry.published_at {
            let cutoff = Utc::now() - chrono::Duration::hours(MAX_STOCK_NEWS_AGE_HOURS);
            if pub_at < cutoff { return "too_old"; }
        }

        let exists: Option<(bool,)> = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM stock_news WHERE content_hash = $1)"
        )
        .bind(&entry.content_hash)
        .fetch_optional(&self.db)
        .await
        .ok()
        .flatten();

        if let Some((true,)) = exists { return "duplicate"; }

        let impact_level = if entry.tickers.len() >= 3 {
            "high"
        } else if !entry.tickers.is_empty() {
            "medium"
        } else {
            "low"
        };

        let published_at = entry.published_at.map(|d| d.to_rfc3339()).unwrap_or_default();
        let tickers_str = entry.tickers.join(",");

        let res = sqlx::query(
            "INSERT INTO stock_news \
             (content_hash, original_url, title, source_name, category, \
              tickers, sentiment, impact_level, is_processed, processed_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, TRUE, NOW()) \
             ON CONFLICT (content_hash) DO NOTHING"
        )
        .bind(&entry.content_hash)
        .bind(&entry.link)
        .bind(&entry.title)
        .bind(&entry.source_name)
        .bind(&entry.category)
        .bind(&tickers_str)
        .bind("neutral")
        .bind(impact_level)
        .execute(&self.db)
        .await;

        if let Err(e) = res {
            warn!(error = %e, "stock db insert failed");
        }

        let summary_truncated = if entry.content.len() > 1000 {
            entry.content[..1000].to_string()
        } else {
            entry.content.clone()
        };

        let equity_data = EquityNewsData {
            id: entry.content_hash.clone(),
            title: entry.title.clone(),
            summary: Some(summary_truncated),
            content: None,
            source_name: entry.source_name.clone(),
            source_url: entry.link.clone(),
            url: entry.link.clone(),
            category: entry.category.clone(),
            tickers: entry.tickers.clone(),
            sentiment: Some("neutral".to_string()),
            impact_level: Some(impact_level.to_string()),
            published_at: if published_at.is_empty() { None } else { Some(published_at) },
            processed_at: Utc::now().to_rfc3339(),
        };

        let embed = ws::build_equity_embed(&equity_data);
        let data = serde_json::json!({
            "article": equity_data,
            "discord_embed": embed,
            "asset_type": "equity",
        });
        let count = self.hub.broadcast(ws::EVENT_EQUITY_NEWS_NEW, data, "equity_news").await;

        let title_short = if entry.title.len() > 50 { &entry.title[..50] } else { &entry.title };
        info!(clients = count, title = title_short, tickers = ?entry.tickers, "stock broadcast ok");

        "processed"
    }
}
