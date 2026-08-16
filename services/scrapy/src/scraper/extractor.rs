use crate::models::News;
use scraper::{Html, Selector};
use serde_json::Value;

pub fn empty_news(url: &str) -> News {
    News {
        title: None,
        author: None,
        published_time: None,
        content: None,
        url: url.to_owned(),
    }
}

pub fn meta(document: &Html, property: &str) -> Option<String> {
    let selector = Selector::parse(&format!("meta[property='{property}']")).ok()?;
    document
        .select(&selector)
        .next()
        .and_then(|node| node.value().attr("content"))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

pub fn text(document: &Html, selector_text: &str) -> Option<String> {
    let selector = Selector::parse(selector_text).ok()?;
    let value = document
        .select(&selector)
        .flat_map(|node| node.text())
        .collect::<Vec<_>>()
        .join(" ");
    let value = value.split_whitespace().collect::<Vec<_>>().join(" ");
    (!value.is_empty()).then_some(value)
}

pub fn json_ld(document: &Html) -> Vec<Value> {
    let selector = Selector::parse("script[type='application/ld+json']").unwrap();
    document
        .select(&selector)
        .filter_map(|node| serde_json::from_str(node.text().collect::<String>().trim()).ok())
        .collect()
}

pub fn set_json_ld_fields(value: &Value, news: &mut News) {
    let Some(object) = value.as_object() else {
        return;
    };
    if news.title.is_none() {
        news.title = object
            .get("headline")
            .or_else(|| object.get("name"))
            .and_then(Value::as_str)
            .map(str::to_owned);
    }

    if news.author.is_none() {
        news.author = object.get("author").and_then(|author| match author {
            Value::String(name) => Some(name.clone()),
            Value::Object(author) => author
                .get("name")
                .and_then(Value::as_str)
                .map(str::to_owned),
            _ => None,
        });
    }

    if news.published_time.is_none() {
        news.published_time = object
            .get("datePublished")
            .and_then(Value::as_str)
            .map(str::to_owned);
    }
    if news.content.is_none() {
        news.content = object
            .get("articleBody")
            .and_then(Value::as_str)
            .map(str::to_owned);
    }
}
