use std::time::Duration;

use atlsd_common::circuit_breaker::CircuitBreaker;
use once_cell::sync::Lazy;
use serde::Deserialize;
use tracing::debug;

static AI_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .unwrap_or_default()
});

static AI_URL: Lazy<String> = Lazy::new(|| {
    std::env::var("AI_SERVICE_URL").unwrap_or_else(|_| "http://localhost:5000".to_string())
});

static AI_CIRCUIT: Lazy<CircuitBreaker> =
    Lazy::new(|| CircuitBreaker::new(5, Duration::from_secs(30), 2));

#[derive(Deserialize)]
struct SentimentRes {
    sentiment: String,
    #[allow(dead_code)]
    score: f64,
}

pub async fn analyze(text: &str) -> String {
    if !AI_CIRCUIT.allow_request().await {
        debug!("FinBERT circuit open, using regex/keyword fallback");
        return fallback_analyze(text);
    }

    let payload = serde_json::json!({ "text": text });
    let url = format!("{}/analyze", *AI_URL);
    let res = AI_CLIENT.post(&url).json(&payload).send().await;

    match res {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<SentimentRes>().await {
                    Ok(data) => {
                        let s = data.sentiment.to_lowercase();
                        if s == "positive" || s == "negative" || s == "neutral" || s == "mixed" {
                            AI_CIRCUIT.record_success().await;
                            return s;
                        }
                        AI_CIRCUIT.record_failure().await;
                    }
                    Err(e) => {
                        debug!(error = %e, "FinBERT response parse failed, using regex/keyword fallback");
                        AI_CIRCUIT.record_failure().await;
                    }
                }
            } else {
                debug!(status = %response.status(), "FinBERT returned non-success status, using regex/keyword fallback");
                AI_CIRCUIT.record_failure().await;
            }
            fallback_analyze(text)
        }
        Err(e) => {
            debug!(error = %e, "FinBERT service unreachable, using regex/keyword fallback");
            AI_CIRCUIT.record_failure().await;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_detects_positive_negative_and_neutral_text() {
        assert_eq!(fallback_analyze("profit growth rally higher"), "positive");
        assert_eq!(fallback_analyze("loss crash decline lower"), "negative");
        assert_eq!(
            fallback_analyze("central bank holds policy steady"),
            "neutral"
        );
    }

    #[test]
    fn fallback_returns_neutral_when_signals_tie() {
        assert_eq!(fallback_analyze("profit loss"), "neutral");
    }
}
