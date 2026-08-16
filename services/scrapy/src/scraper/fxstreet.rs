use regex::Regex;
use scraper::Html;
use serde_json::Value;
use std::sync::OnceLock;

use super::extractor::{empty_news, json_ld, meta, set_json_ld_fields};
use crate::{error::Result, models::News};

pub struct FxStreetExtractor;

impl FxStreetExtractor {
    pub fn extract(html: &str, url: &str) -> Result<News> {
        let mut news = empty_news(url);
        let document = Html::parse_document(html);

        for payload in next_payloads(html) {
            if let Ok(value) = serde_json::from_str::<Value>(&payload) {
                extract_json(&value, &mut news);
            }
        }
        for value in json_ld(&document) {
            set_json_ld_fields(&value, &mut news);
        }

        news.title = news.title.or_else(|| meta(&document, "og:title"));
        news.published_time = news
            .published_time
            .or_else(|| meta(&document, "article:published_time"));
        news.content = news.content.or_else(|| meta(&document, "og:description"));
        Ok(news)
    }
}

fn next_payloads(html: &str) -> impl Iterator<Item = String> + '_ {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    let regex =
        REGEX.get_or_init(|| Regex::new(r#"self\.__next_f\.push\(\[1,"(.*?)"\]\)"#).unwrap());
    regex.captures_iter(html).filter_map(|capture| {
        capture.get(1).map(|value| {
            value
                .as_str()
                .replace("\\\"", "\"")
                .replace("\\n", "\n")
                .replace("\\\\", "\\")
        })
    })
}

fn extract_json(value: &Value, news: &mut News) {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                match key.as_str() {
                    "headline" | "title" if news.title.is_none() => {
                        news.title = value.as_str().map(str::to_owned)
                    }
                    "author" if news.author.is_none() => {
                        news.author = value.as_str().map(str::to_owned).or_else(|| {
                            value.get("name").and_then(Value::as_str).map(str::to_owned)
                        });
                    }
                    "datePublished" | "publishedTime" | "published_time"
                        if news.published_time.is_none() =>
                    {
                        news.published_time = value.as_str().map(str::to_owned)
                    }
                    "articleBody" | "content" | "body" if news.content.is_none() => {
                        news.content = value.as_str().map(str::to_owned)
                    }
                    _ => {}
                }
                extract_json(value, news);
            }
        }
        Value::Array(items) => items.iter().for_each(|item| extract_json(item, news)),
        _ => {}
    }
}
