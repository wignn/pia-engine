use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreEvent {
    pub event: String,
    pub data: Option<serde_json::Value>,
    pub channel: Option<String>,
    pub timestamp: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArticleData {
    pub id: String,
    pub title: String,
    pub title_id: Option<String>,
    pub summary: Option<String>,
    pub summary_id: Option<String>,
    pub source_name: String,
    #[serde(default)]
    pub original_url: Option<String>,
    pub sentiment: Option<String>,
    pub impact_level: Option<String>,
    #[serde(default)]
    pub currency_pairs: Vec<String>,
    pub published_at: Option<String>,
    pub processed_at: Option<String>,
    pub image_url: Option<String>,
}

impl ArticleData {
    pub fn from_value(mut value: serde_json::Value) -> Result<Self, serde_json::Error> {
        if let Some(object) = value.as_object_mut() {
            if object.contains_key("title_id") {
                object.remove("translated_title");
            } else if let Some(title_id) = object.remove("translated_title") {
                object.insert("title_id".to_string(), title_id);
            }
            if object.contains_key("original_url") {
                object.remove("url");
            } else if let Some(url) = object.remove("url") {
                object.insert("original_url".to_string(), url);
            }
        }
        serde_json::from_value(value)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiscordEmbed {
    pub title: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub color: Option<u32>,
    pub fields: Option<Vec<EmbedField>>,
    pub thumbnail: Option<EmbedMedia>,
    pub image: Option<EmbedMedia>,
    pub footer: Option<EmbedFooter>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmbedField {
    pub name: String,
    pub value: String,
    #[serde(default)]
    pub inline: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmbedMedia {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmbedFooter {
    pub text: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CalendarEventData {
    pub event_id: String,
    pub title: String,
    pub currency: String,
    pub date_wib: String,
    pub impact: String,
    pub forecast: String,
    pub previous: String,
    pub minutes_until: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TweetData {
    pub id: String,
    pub text: String,
    pub author_username: String,
    pub author_name: String,
    pub author_avatar: Option<String>,
    pub created_at: Option<String>,
    pub url: String,
    #[serde(default)]
    pub media_urls: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::ArticleData;

    #[test]
    fn parses_news_service_article_without_embed_or_url() {
        let article = ArticleData::from_value(serde_json::json!({
            "id": "article-1",
            "title": "Headline",
            "original_title": "Original",
            "url": "https://example.test",
            "original_url": "https://example.test",
            "summary": "Summary",
            "source_name": "Wire",
            "impact_level": "high"
        }))
        .unwrap();

        assert_eq!(article.title, "Headline");
        assert_eq!(
            article.original_url.as_deref(),
            Some("https://example.test")
        );
    }
}
