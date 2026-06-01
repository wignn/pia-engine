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
    #[serde(alias = "original_title")]
    pub title: String,
    #[serde(alias = "translated_title")]
    pub title_id: Option<String>,
    pub summary: Option<String>,
    pub summary_id: Option<String>,
    pub source_name: String,
    #[serde(alias = "url")]
    pub original_url: String,
    pub sentiment: Option<String>,
    pub impact_level: Option<String>,
    #[serde(default)]
    pub currency_pairs: Vec<String>,
    pub published_at: Option<String>,
    pub processed_at: Option<String>,
    pub image_url: Option<String>,
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
