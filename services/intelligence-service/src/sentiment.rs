use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::state::AppState;

#[derive(Deserialize, Serialize)]
pub struct AnalyzeRequest {
    pub text: String,
}

pub async fn analyze_text(
    State(state): State<AppState>,
    Json(payload): Json<AnalyzeRequest>,
) -> Json<Value> {
    let text_trimmed = payload.text.trim();
    if text_trimmed.is_empty() {
        return Json(json!({ "error": "text is required" }));
    }

    let is_url = text_trimmed.starts_with("http://") || text_trimmed.starts_with("https://");
    if is_url {
        let cached: Option<(String, String, Value)> = sqlx::query_as(
            "SELECT title, content, raw_response FROM url_analysis_cache WHERE url = $1",
        )
        .bind(text_trimmed)
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None);

        if let Some((_title, _content, raw_response)) = cached {
            return Json(raw_response);
        }
    }

    let url = format!(
        "{}/analyze",
        state.config.ai_service_url.trim_end_matches('/')
    );
    let analyzer_payload = if is_url {
        json!({ "url": text_trimmed })
    } else {
        json!({ "text": text_trimmed })
    };

    match state.http.post(&url).json(&analyzer_payload).send().await {
        Ok(res) if res.status().is_success() => match res.json::<Value>().await {
            Ok(val) => {
                if is_url {
                    let title = val.get("title").and_then(|t| t.as_str()).unwrap_or("");
                    let content = val.get("content").and_then(|c| c.as_str()).unwrap_or("");
                    let _ = sqlx::query(
                        "INSERT INTO url_analysis_cache (url, title, content, raw_response) VALUES ($1, $2, $3, $4) ON CONFLICT (url) DO UPDATE SET raw_response = EXCLUDED.raw_response, title = EXCLUDED.title, content = EXCLUDED.content",
                    )
                    .bind(text_trimmed)
                    .bind(title)
                    .bind(content)
                    .bind(&val)
                    .execute(&state.db)
                    .await;
                }
                Json(val)
            }
            Err(err) => {
                Json(json!({ "error": format!("Failed to parse analyzer response: {err}") }))
            }
        },
        Ok(res) => Json(
            json!({ "error": format!("Analyzer service returned error status: {}", res.status()) }),
        ),
        Err(_) if !is_url => {
            Json(json!({ "sentiment": fallback_analyze(text_trimmed), "source": "fallback" }))
        }
        Err(err) => {
            Json(json!({ "error": format!("Failed to connect to analyzer service: {err}") }))
        }
    }
}

fn fallback_analyze(text: &str) -> String {
    let text_lower = text.to_lowercase();
    let pos = [
        "surge", "gain", "bullish", "rise", "growth", "rally", "profit", "higher", "positive",
    ];
    let neg = [
        "plunge", "loss", "bearish", "drop", "fall", "crash", "decline", "lower", "negative",
    ];
    let pos_count = pos
        .iter()
        .filter(|word| text_lower.contains(**word))
        .count();
    let neg_count = neg
        .iter()
        .filter(|word| text_lower.contains(**word))
        .count();
    if pos_count > neg_count {
        "positive".to_string()
    } else if neg_count > pos_count {
        "negative".to_string()
    } else {
        "neutral".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_detects_sentiment() {
        assert_eq!(fallback_analyze("profit growth rally higher"), "positive");
        assert_eq!(fallback_analyze("loss crash decline lower"), "negative");
        assert_eq!(
            fallback_analyze("central bank holds policy steady"),
            "neutral"
        );
    }
}
