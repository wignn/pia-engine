use axum::{extract::Query, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::warn;

#[derive(Deserialize)]
pub struct CalendarQuery {
    /// Filter by impact level: "high", "medium", "low". Defaults to "high".
    pub impact: Option<String>,
    /// Maximum number of events to return (1–25). Defaults to 10.
    pub limit: Option<usize>,
}

/// GET /api/v1/forex/calendar
/// Fetches upcoming high-impact economic events from Forex Factory.
/// Query params:
///   - impact=high|medium|low  (default: high)
///   - limit=1..25             (default: 10)
pub async fn list_calendar(Query(query): Query<CalendarQuery>) -> Json<Value> {
    let impact_filter = query
        .impact
        .as_deref()
        .unwrap_or("high")
        .to_lowercase();
    let limit = query.limit.unwrap_or(10).clamp(1, 25);

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .user_agent("Mozilla/5.0 (compatible; AtlsdBot/1.0)")
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            warn!(error = %e, "failed to build reqwest client");
            return Json(json!({ "error": "internal client error" }));
        }
    };

    let response = client
        .get("https://nfs.faireconomy.media/ff_calendar_thisweek.json")
        .send()
        .await;

    let body = match response {
        Ok(resp) => match resp.text().await {
            Ok(text) => text,
            Err(e) => {
                warn!(error = %e, "failed to read calendar response body");
                return Json(json!({ "error": "upstream read error" }));
            }
        },
        Err(e) => {
            warn!(error = %e, "failed to fetch forex factory calendar");
            return Json(json!({ "error": "upstream request failed" }));
        }
    };

    let events: Value = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(e) => {
            warn!(error = %e, "failed to parse calendar JSON");
            return Json(json!({ "error": "upstream parse error" }));
        }
    };

    let arr = match events.as_array() {
        Some(a) => a,
        None => return Json(json!({ "error": "unexpected calendar format" })),
    };

    let mut items: Vec<Value> = arr
        .iter()
        .filter(|ev| {
            let impact = ev["impact"].as_str().unwrap_or("").to_lowercase();
            match impact_filter.as_str() {
                "high"   => impact.contains("high") || impact == "red",
                "medium" => impact.contains("medium") || impact == "orange",
                "low"    => impact.contains("low") || impact == "yellow",
                _        => true,
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
    }))
}
