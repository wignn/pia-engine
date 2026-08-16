use scraper::Html;

use crate::{error::Result, models::News};

use super::extractor::{empty_news, json_ld, meta, set_json_ld_fields, text};

pub struct InvestingLiveExtractor;

impl InvestingLiveExtractor {
    pub fn extract(html: &str, url: &str) -> Result<News> {
        let document = Html::parse_document(html);

        let mut news = empty_news(url);

        for value in json_ld(&document) {
            set_json_ld_fields(&value, &mut news);
        }

        news.title = news
            .title
            .or_else(|| text(&document, "h1"))
            .or_else(|| meta(&document, "og:title"));
        news.content = news
            .content
            .or_else(|| text(&document, "article p"))
            .or_else(|| text(&document, "article"))
            .or_else(|| text(&document, "[data-test='article-content']"))
            .or_else(|| meta(&document, "og:description"));
        news.author = news
            .author
            .or_else(|| text(&document, "[class*='author']"))
            .or_else(|| text(&document, "a[href*='author']"));
        news.published_time = news
            .published_time
            .or_else(|| meta(&document, "article:published_time"));
        Ok(news)
    }
}
