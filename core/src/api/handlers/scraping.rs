use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use std::time::Duration;

use crate::api::state::AppState;
use crate::scraper::article::ArticleScraper;

#[derive(Deserialize)]
pub struct ScrapeRequest {
    pub link: String,
}

pub async fn scrape_article(
    State(state): State<AppState>,
    Json(body): Json<ScrapeRequest>,
) -> Json<Value> {
    if body.link.is_empty() {
        return Json(json!({ "error": "link is required" }));
    }

    let scraper = ArticleScraper::new("Mozilla/5.0", Duration::from_secs(10));

    match scraper.scrape(&body.link).await {
        Ok(article) => Json(json!({
            "title": article.title,
            "content": article.content,
            "published_at": article.published_at,
            "tags": article.tags,
        })),
        Err(e) => Json(json!({
            "error": format!("Failed to scrape article: {}", e),
        })),
    }
}
