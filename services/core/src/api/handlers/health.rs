use axum::Json;
use serde_json::{json, Value};

pub async fn health() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "service": "world-info",
    }))
}

pub async fn root() -> Json<Value> {
    Json(json!({
        "service": "World Info Server (Rust)",
        "version": "1.0.0",
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn health_returns_service_status() {
        let Json(body) = health().await;

        assert_eq!(body["status"], "healthy");
        assert_eq!(body["service"], "world-info");
    }

    #[tokio::test]
    async fn root_returns_service_metadata() {
        let Json(body) = root().await;

        assert_eq!(body["service"], "World Info Server (Rust)");
        assert_eq!(body["version"], "1.0.0");
    }
}
