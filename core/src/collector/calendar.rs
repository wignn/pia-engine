use chrono::{DateTime, FixedOffset, Utc};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

const FOREX_FACTORY_URL: &str = "https://nfs.faireconomy.media/ff_calendar_thisweek.json";
const CACHE_TTL: Duration = Duration::from_secs(2 * 3600);

fn wib_offset() -> FixedOffset {
    FixedOffset::east_opt(7 * 3600).unwrap()
}

#[derive(Debug, Clone)]
pub struct CalendarEvent {
    pub title: String,
    pub country: String,
    pub currency: String,
    pub date_utc: DateTime<Utc>,
    pub date_wib: String,
    pub impact: String,
    pub forecast: String,
    pub previous: String,
    pub event_id: String,
    pub minutes_until: i64,
}

#[derive(Debug, Deserialize)]
struct RawCalendarEvent {
    title: Option<String>,
    country: Option<String>,
    date: Option<String>,
    impact: Option<String>,
    forecast: Option<String>,
    previous: Option<String>,
}

static CURRENCY_MAP: &[(&str, &str)] = &[
    ("USD", "USD 🇺🇸"),
    ("EUR", "EUR 🇪🇺"),
    ("GBP", "GBP 🇬🇧"),
    ("JPY", "JPY 🇯🇵"),
    ("CHF", "CHF 🇨🇭"),
    ("AUD", "AUD 🇦🇺"),
    ("NZD", "NZD 🇳🇿"),
    ("CAD", "CAD 🇨🇦"),
    ("CNY", "CNY 🇨🇳"),
];

pub struct CalendarCollector {
    client: Client,
    cache: RwLock<Option<(Vec<CalendarEvent>, Instant)>>,
}

impl CalendarCollector {
    pub fn new(timeout: Duration) -> Self {
        Self {
            client: Client::builder()
                .timeout(timeout)
                .build()
                .expect("failed to build HTTP client"),
            cache: RwLock::new(None),
        }
    }

    pub async fn fetch_events(&self, force_refresh: bool) -> Result<Vec<CalendarEvent>, String> {
        if !force_refresh {
            if let Ok(guard) = self.cache.read() {
                if let Some((events, cached_at)) = guard.as_ref() {
                    if cached_at.elapsed() < CACHE_TTL {
                        debug!(count = events.len(), "using cached calendar events");
                        return Ok(events.clone());
                    }
                }
            }
        }

        let resp = self
            .client
            .get(FOREX_FACTORY_URL)
            .send()
            .await
            .map_err(|e| format!("fetch calendar: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            return self
                .stale_or_empty()
                .ok_or_else(|| format!("calendar returned {}", status));
        }

        let raw_events: Vec<RawCalendarEvent> = resp
            .json()
            .await
            .map_err(|e| format!("decode calendar: {}", e))?;

        let events: Vec<CalendarEvent> = raw_events
            .into_iter()
            .filter_map(parse_calendar_event)
            .collect();

        if let Ok(mut guard) = self.cache.write() {
            *guard = Some((events.clone(), Instant::now()));
        }

        info!(count = events.len(), "fetched calendar events");
        Ok(events)
    }

    pub async fn get_upcoming_high_impact(
        &self,
        minutes_before: i64,
        minutes_window: i64,
    ) -> Result<Vec<CalendarEvent>, String> {
        let events = self.fetch_events(false).await?;
        let now = Utc::now();
        let mut upcoming = Vec::new();

        for mut ev in events {
            let impact = ev.impact.to_lowercase();
            if impact != "high" && impact != "red" {
                continue;
            }

            let mins = (ev.date_utc - now).num_minutes();
            let min_bound = minutes_before - minutes_window;
            let max_bound = minutes_before;

            if mins >= min_bound && mins <= max_bound {
                ev.minutes_until = mins;
                upcoming.push(ev);
            }
        }

        if !upcoming.is_empty() {
            let titles: Vec<&str> = upcoming.iter().map(|e| e.title.as_str()).collect();
            info!(count = upcoming.len(), events = ?titles, "upcoming high-impact events");
        }

        Ok(upcoming)
    }

    fn stale_or_empty(&self) -> Option<Vec<CalendarEvent>> {
        if let Ok(guard) = self.cache.read() {
            if let Some((events, _)) = guard.as_ref() {
                warn!("returning stale calendar cache");
                return Some(events.clone());
            }
        }
        None
    }
}

fn parse_calendar_event(raw: RawCalendarEvent) -> Option<CalendarEvent> {
    let title = raw.title?.trim().to_string();
    let country = raw.country.unwrap_or_default().trim().to_string();
    let date_str = raw.date?;

    if title.is_empty() || date_str.is_empty() {
        return None;
    }

    let date_utc = date_str
        .parse::<DateTime<Utc>>()
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%dT%H:%M:%S")
                .map(|ndt| ndt.and_utc())
        })
        .ok()?;

    let wib = wib_offset();
    let date_wib = date_utc.with_timezone(&wib);
    let date_wib_str = format!("{} WIB", date_wib.format("%d/%m %H:%M"));

    let currency_map: HashMap<&str, &str> = CURRENCY_MAP.iter().copied().collect();
    let currency = currency_map
        .get(country.to_uppercase().as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| country.clone());

    let forecast = raw.forecast.unwrap_or_default().trim().to_string();
    let forecast = if forecast.is_empty() { "—".to_string() } else { forecast };

    let previous = raw.previous.unwrap_or_default().trim().to_string();
    let previous = if previous.is_empty() { "—".to_string() } else { previous };

    let truncated_title = if title.len() > 30 { &title[..30] } else { &title };
    let event_id = format!("{}_{}_{}", date_str, country, truncated_title);

    Some(CalendarEvent {
        title,
        country,
        currency,
        date_utc,
        date_wib: date_wib_str,
        impact: raw.impact.unwrap_or_default().trim().to_string(),
        forecast,
        previous,
        event_id,
        minutes_until: 0,
    })
}
