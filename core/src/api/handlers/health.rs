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
