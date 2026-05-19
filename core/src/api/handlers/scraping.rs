use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use std::time::Duration;

use crate::api::state::AppState;
use crate::scraper::article::ArticleScraper;
use crate::tenant::context::TenantContext;

#[derive(Deserialize)]
pub struct ScrapeRequest {
    pub link: String,
}

pub async fn scrape_article(
    State(state): State<AppState>,
    request: axum::extract::Request,
) -> Json<Value> {
    // The tenant context is injected by strict API-key middleware.
    if let Some(ctx) = request.extensions().get::<TenantContext>() {
        if !ctx.can_scrape {
            return Json(json!({ "error": "Scraping requires Starter plan or higher" }));
        }
    }

    // Parse the body after reading extension-backed authorization context.
    let body_bytes = match axum::body::to_bytes(request.into_body(), 1024 * 16).await {
        Ok(b) => b,
        Err(_) => return Json(json!({ "error": "invalid request body" })),
    };
    let body: ScrapeRequest = match serde_json::from_slice(&body_bytes) {
        Ok(b) => b,
        Err(_) => return Json(json!({ "error": "invalid JSON body" })),
    };
    if body.link.is_empty() {
        return Json(json!({ "error": "link is required" }));
    }

    let scraper = ArticleScraper::new(
        &state.config.scraper_ua,
        Duration::from_secs(state.config.scraper_timeout),
    );

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
