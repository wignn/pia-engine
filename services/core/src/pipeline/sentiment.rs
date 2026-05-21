use once_cell::sync::Lazy;
use serde::Deserialize;
use tracing::debug;

static AI_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2)) // Low timeout to prevent pipeline blocking
        .build()
        .unwrap_or_default()
});

static AI_URL: Lazy<String> = Lazy::new(|| {
    std::env::var("AI_SERVICE_URL").unwrap_or_else(|_| "http://localhost:5000".to_string())
});

#[derive(Deserialize)]
struct SentimentRes {
    sentiment: String,
    #[allow(dead_code)]
    score: f64,
}

pub async fn analyze(text: &str) -> String {
    let payload = serde_json::json!({ "text": text });
    let url = format!("{}/analyze", *AI_URL);
    let res = AI_CLIENT.post(&url).json(&payload).send().await;

    match res {
        Ok(response) => {
            if response.status().is_success() {
                if let Ok(data) = response.json::<SentimentRes>().await {
                    let s = data.sentiment.to_lowercase();

                    if s == "positive" || s == "negative" || s == "neutral" || s == "mixed" {
                        return s;
                    }
                }
            }
            fallback_analyze(text)
        }
        Err(e) => {
            debug!(error = %e, "FinBERT service unreachable, using regex/keyword fallback");
            fallback_analyze(text)
        }
    }
}

fn fallback_analyze(text: &str) -> String {
    let text_lower = text.to_lowercase();

    // Positive signals
    let pos_signals = [
        "surge",
        "gain",
        "bullish",
        "upbeat",
        "rise",
        "rising",
        "growth",
        "rally",
        "profit",
        "succeed",
        "higher",
        "positive",
        "beat expectation",
        "exceed",
    ];

    // Negative signals
    let neg_signals = [
        "plunge",
        "loss",
        "bearish",
        "drop",
        "falling",
        "fall",
        "crash",
        "decline",
        "slump",
        "deficit",
        "lower",
        "negative",
        "miss expectation",
        "fail",
    ];

    let mut pos_count = 0;
    let mut neg_count = 0;

    for word in pos_signals.iter() {
        if text_lower.contains(word) {
            pos_count += 1;
        }
    }

    for word in neg_signals.iter() {
        if text_lower.contains(word) {
            neg_count += 1;
        }
    }

    if pos_count > neg_count {
        "positive".to_string()
    } else if neg_count > pos_count {
        "negative".to_string()
    } else {
        "neutral".to_string()
    }
}
