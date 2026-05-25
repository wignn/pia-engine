use std::sync::RwLock;
use std::time::{Duration, Instant};

use axum::{extract::Query, Json};
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::{debug, warn};

const FOREX_FACTORY_URL: &str = "https://nfs.faireconomy.media/ff_calendar_thisweek.json";
const CALENDAR_CACHE_TTL: Duration = Duration::from_secs(30 * 60);

static CALENDAR_CACHE: Lazy<RwLock<Option<(Value, Instant)>>> = Lazy::new(|| RwLock::new(None));
static CALENDAR_BACKOFF_UNTIL: Lazy<RwLock<Option<Instant>>> = Lazy::new(|| RwLock::new(None));

#[derive(Deserialize)]
pub struct CalendarQuery {
    /// Filter by impact level: "high", "medium", "low". Defaults to "high".
    pub impact: Option<String>,
    /// Maximum number of events to return (1–25). Defaults to 10.
    pub limit: Option<usize>,
}

pub async fn list_calendar(Query(query): Query<CalendarQuery>) -> Json<Value> {
    let impact_filter = query.impact.as_deref().unwrap_or("high").to_lowercase();
    let limit = query.limit.unwrap_or(10).clamp(1, 25);

    let (events, cache_status) = match get_calendar_events().await {
        Ok(result) => result,
        Err(error) => return Json(json!({ "error": error })),
    };

    let arr = match events.as_array() {
        Some(a) => a,
        None => return Json(json!({ "error": "unexpected calendar format" })),
    };

    let items: Vec<Value> = arr
        .iter()
        .filter(|ev| {
            let impact = ev["impact"].as_str().unwrap_or("").to_lowercase();
            match impact_filter.as_str() {
                "high" => impact.contains("high") || impact == "red",
                "medium" => impact.contains("medium") || impact == "orange",
                "low" => impact.contains("low") || impact == "yellow",
                _ => true,
            }
        })
        .take(limit)
        .map(|ev| {
            let country = ev["country"].as_str().unwrap_or("");
            json!({
                "title":    ev["title"].as_str().unwrap_or(""),
                "currency": country,
                "date":     ev["date"].as_str().unwrap_or(""),
                "impact":   ev["impact"].as_str().unwrap_or(""),
                "forecast": ev["forecast"].as_str().unwrap_or(""),
                "previous": ev["previous"].as_str().unwrap_or(""),
                "actual":   ev["actual"].as_str().unwrap_or(""),
            })
        })
        .collect();

    Json(json!({
        "items": items,
        "total": items.len(),
        "filter": { "impact": impact_filter, "limit": limit },
        "source": "forexfactory",
        "cache": { "status": cache_status },
    }))
}

async fn get_calendar_events() -> Result<(Value, &'static str), String> {
    if let Ok(guard) = CALENDAR_CACHE.read() {
        if let Some((events, cached_at)) = guard.as_ref() {
            if cached_at.elapsed() < CALENDAR_CACHE_TTL {
                debug!("using cached calendar response");
                return Ok((events.clone(), "hit"));
            }
        }
    }

    if let Ok(guard) = CALENDAR_BACKOFF_UNTIL.read() {
        if let Some(until) = *guard {
            if Instant::now() < until {
                if let Some(events) = stale_calendar_events() {
                    return Ok((events, "stale"));
                }
            }
        }
    }

    match fetch_calendar_events().await {
        Ok(events) => {
            if let Ok(mut guard) = CALENDAR_CACHE.write() {
                *guard = Some((events.clone(), Instant::now()));
            }
            if let Ok(mut guard) = CALENDAR_BACKOFF_UNTIL.write() {
                *guard = None;
            }
            Ok((events, "miss"))
        }
        Err(error) => {
            if let Ok(mut guard) = CALENDAR_BACKOFF_UNTIL.write() {
                *guard = Some(Instant::now() + Duration::from_secs(15 * 60));
            }
            if let Some(events) = stale_calendar_events() {
                warn!(error = %error, "returning stale forex factory calendar cache");
                Ok((events, "stale"))
            } else {
                Err(error)
            }
        }
    }
}

async fn fetch_calendar_events() -> Result<Value, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .map_err(|error| format!("internal client error: {error}"))?;

    let response = client
        .get(FOREX_FACTORY_URL)
        .send()
        .await
        .map_err(|error| format!("upstream request failed: {error}"))?;

    let status = response.status();
    if !status.is_success() {
        warn!(status = %status, "forex factory calendar request returned non-success status");
        return Err(format!("upstream returned status: {status}"));
    }

    response
        .json::<Value>()
        .await
        .map_err(|error| format!("upstream parse error: {error}"))
}

fn stale_calendar_events() -> Option<Value> {
    CALENDAR_CACHE
        .read()
        .ok()
        .and_then(|guard| guard.as_ref().map(|(events, _)| events.clone()))
}
