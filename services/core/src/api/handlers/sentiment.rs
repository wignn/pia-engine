use crate::api::state::AppState;
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Deserialize, Serialize)]
pub struct AnalyzeRequest {
    pub text: String,
}

pub async fn analyze_text(
    State(_state): State<AppState>,
    Json(payload): Json<AnalyzeRequest>,
) -> Json<Value> {
    let text_trimmed = payload.text.trim();
    if text_trimmed.is_empty() {
        return Json(json!({ "error": "text is required" }));
    }

    let is_url = text_trimmed.starts_with("http://") || text_trimmed.starts_with("https://");

    if is_url {
        let cached: Option<(String, String, Value)> = sqlx::query_as(
            "SELECT title, content, raw_response FROM url_analysis_cache WHERE url = $1"
        )
        .bind(text_trimmed)
        .fetch_optional(&_state.db)
        .await
        .unwrap_or(None);

        if let Some((_title, _content, raw_response)) = cached {
            return Json(raw_response);
        }
    }

    let ai_url =
        std::env::var("AI_SERVICE_URL").unwrap_or_else(|_| "http://localhost:5000".to_string());
    let url = format!("{}/analyze", ai_url);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30)) // Increased timeout to accommodate scraping
        .build()
        .unwrap_or_default();

    let analyzer_payload = if is_url {
        json!({ "url": text_trimmed })
    } else {
        json!({ "text": text_trimmed })
    };

    match client.post(&url).json(&analyzer_payload).send().await {
        Ok(res) => {
            if res.status().is_success() {
                match res.json::<Value>().await {
                    Ok(val) => {
                        if is_url {
                            let title = val.get("title").and_then(|t| t.as_str()).unwrap_or("");
                            let content = val.get("content").and_then(|c| c.as_str()).unwrap_or("");
                            
                            let _ = sqlx::query(
                                "INSERT INTO url_analysis_cache (url, title, content, raw_response) \
                                 VALUES ($1, $2, $3, $4) \
                                 ON CONFLICT (url) DO UPDATE SET raw_response = EXCLUDED.raw_response, title = EXCLUDED.title, content = EXCLUDED.content"
                            )
                            .bind(text_trimmed)
                            .bind(title)
                            .bind(content)
                            .bind(&val)
                            .execute(&_state.db)
                            .await;
                        }
                        Json(val)
                    }
                    Err(e) => Json(
                        json!({ "error": format!("Failed to parse analyzer response: {}", e) }),
                    ),
                }
            } else {
                Json(
                    json!({ "error": format!("Analyzer service returned error status: {}", res.status()) }),
                )
            }
        }
        Err(e) => Json(json!({ "error": format!("Failed to connect to analyzer service: {}", e) })),
    }
}
