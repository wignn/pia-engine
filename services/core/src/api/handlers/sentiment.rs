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
    if payload.text.trim().is_empty() {
        return Json(json!({ "error": "text is required" }));
    }

    let ai_url =
        std::env::var("AI_SERVICE_URL").unwrap_or_else(|_| "http://localhost:5000".to_string());
    let url = format!("{}/analyze", ai_url);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap_or_default();

    match client.post(&url).json(&payload).send().await {
        Ok(res) => {
            if res.status().is_success() {
                match res.json::<Value>().await {
                    Ok(val) => Json(val),
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
