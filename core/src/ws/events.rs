use chrono::{TimeZone, Utc};
use chrono_tz::Asia::Jakarta;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

// Event type constants
pub const EVENT_NEWS_NEW: &str = "news.new";
pub const EVENT_NEWS_HIGH_IMPACT: &str = "news.high_impact";
pub const EVENT_EQUITY_NEWS_NEW: &str = "equity.news.new";
pub const EVENT_CALENDAR_REMINDER: &str = "calendar.reminder";
pub const EVENT_MARKET_TRADE: &str = "market.trade";
pub const EVENT_GOLD_VOLATILITY_SPIKE: &str = "gold.volatility_spike";
pub const EVENT_X_NEW: &str = "x.new";
pub const EVENT_HEARTBEAT: &str = "heartbeat";
pub const EVENT_SYSTEM_STATUS: &str = "system.status";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsArticleData {
    pub id: String,
    #[serde(rename = "original_title")]
    pub title: String,
    #[serde(rename = "translated_title", skip_serializing_if = "Option::is_none")]
    pub title_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(rename = "summary_id", skip_serializing_if = "Option::is_none")]
    pub summary_id: Option<String>,
    pub source_name: String,
    pub source_url: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sentiment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub impact_level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at: Option<String>,
    pub processed_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
}

pub fn build_news_embed(a: &NewsArticleData) -> Value {
    let color = 0x0099FF;
    let impact_bar = "▰▰▰";

    let time_str = format_wib(a.published_at.as_deref());
    let footer_date = format_footer_date(a.published_at.as_deref());

    let display_title = a.title_id.as_deref().unwrap_or(&a.title);
    let display_summary = a
        .summary_id
        .as_deref()
        .or(a.summary.as_deref())
        .unwrap_or("");
    let display_summary = truncate_str(display_summary, 300);

    let desc = format!("**MARKET**\n{}\n\n{}", display_title, display_summary);

    json!({
        "title": display_title,
        "description": desc,
        "color": color,
        "fields": [
            { "name": "Waktu", "value": time_str, "inline": true },
            { "name": "Impact", "value": impact_bar, "inline": true },
            { "name": "Sumber", "value": format!("[Baca Selengkapnya]({})", a.url), "inline": false },
        ],
        "footer": {
            "text": format!("Forex Alert • {} • {}", a.source_name, footer_date),
        }
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquityNewsData {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    pub source_name: String,
    pub source_url: String,
    pub url: String,
    pub category: String,
    pub tickers: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sentiment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub impact_level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at: Option<String>,
    pub processed_at: String,
}

pub fn build_equity_embed(s: &EquityNewsData) -> Value {
    let color = 0x5865F2;

    let impact_bars: std::collections::HashMap<&str, &str> = [
        ("high", "▰▰▰"),
        ("medium", "▰▰▱"),
        ("low", "▰▱▱"),
    ]
    .into_iter()
    .collect();
    let impact_bar = s
        .impact_level
        .as_deref()
        .and_then(|l| impact_bars.get(l).copied())
        .unwrap_or("▰▱▱");

    let time_str = format_wib(s.published_at.as_deref());
    let footer_date = format_footer_date(s.published_at.as_deref());

    let cat = if s.category.is_empty() {
        "MARKET"
    } else {
        &s.category
    };

    let summary = truncate_str(s.summary.as_deref().unwrap_or(""), 300);
    let desc = format!("**{}**\n{}\n\n{}", cat, s.title, summary);

    let mut fields = vec![
        json!({ "name": "Waktu", "value": time_str, "inline": true }),
        json!({ "name": "Impact", "value": impact_bar, "inline": true }),
    ];

    if !s.tickers.is_empty() {
        let tickers_str: String = s.tickers.iter().take(5).cloned().collect::<Vec<_>>().join(", ");
        fields.push(json!({ "name": "Tickers", "value": tickers_str, "inline": true }));
    }

    fields.push(json!({
        "name": "Sumber",
        "value": format!("[Baca Selengkapnya]({})", s.url),
        "inline": false,
    }));

    json!({
        "title": s.title,
        "description": desc,
        "color": color,
        "fields": fields,
        "footer": {
            "text": format!("Equity Alert • {} • {}", s.source_name, footer_date),
        }
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XPostData {
    pub id: String,
    pub text: String,
    pub author_username: String,
    pub author_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_avatar: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_urls: Option<Vec<String>>,
}

pub fn build_x_embed(t: &XPostData) -> Value {
    let color = 0x1DA1F2;

    let time_str = format_wib(t.created_at.as_deref());
    let footer_date = format_footer_date(t.created_at.as_deref());

    let text = truncate_str(&t.text, 500);
    let desc = format!("{}\n\n[Lihat di X →]({})", text, t.url);

    let mut result = json!({
        "title": format!("@{}", t.author_username),
        "description": desc,
        "color": color,
        "fields": [
            { "name": "Waktu", "value": time_str, "inline": true },
            { "name": "Author", "value": t.author_name, "inline": true },
        ],
        "footer": {
            "text": format!("X • @{} • {}", t.author_username, footer_date),
        }
    });

    if let Some(avatar) = &t.author_avatar {
        if !avatar.is_empty() {
            result["thumbnail"] = json!({ "url": avatar });
        }
    }

    if let Some(media) = &t.media_urls {
        if let Some(first) = media.first() {
            result["image"] = json!({ "url": first });
        }
    }

    result
}

// --- Helpers ---

fn format_wib(iso_time: Option<&str>) -> String {
    let Some(iso) = iso_time else {
        return "N/A".to_string();
    };
    match iso.parse::<chrono::DateTime<Utc>>() {
        Ok(dt) => dt.with_timezone(&Jakarta).format("%H:%M WIB").to_string(),
        Err(_) => "N/A".to_string(),
    }
}

fn format_footer_date(iso_time: Option<&str>) -> String {
    let Some(iso) = iso_time else {
        return String::new();
    };
    match iso.parse::<chrono::DateTime<Utc>>() {
        Ok(dt) => dt
            .with_timezone(&Jakarta)
            .format("%d/%m/%Y %H:%M")
            .to_string(),
        Err(_) => String::new(),
    }
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}
