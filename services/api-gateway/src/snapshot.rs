use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::Response,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
pub struct CreateSnapshotRequest {
    pub image: String,
    pub symbol: Option<String>,
    pub timeframe: Option<String>,
}

pub async fn create_snapshot(
    State(_state): State<crate::state::AppState>,
    Json(payload): Json<CreateSnapshotRequest>,
) -> Result<Json<Value>, (StatusCode, &'static str)> {
    let raw = payload.image.trim();
    let base64_str = if let Some(idx) = raw.find(";base64,") {
        &raw[idx + 8..]
    } else {
        raw
    };

    use base64::Engine as _;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(base64_str)
        .map_err(|_| (StatusCode::BAD_REQUEST, "invalid base64 image data"))?;

    if bytes.len() > 10 * 1024 * 1024 {
        return Err((StatusCode::PAYLOAD_TOO_LARGE, "image exceeds 10MB limit"));
    }

    let id = format!(
        "s_{}",
        &uuid::Uuid::new_v4().to_string().replace('-', "")[..12]
    );
    let dir = std::path::Path::new("/tmp/pia_snapshots");
    let _ = std::fs::create_dir_all(dir);
    let file_path = dir.join(format!("{}.png", id));
    std::fs::write(&file_path, bytes)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to save snapshot"))?;

    let url = format!("https://api-engine.wign.dev/snapshot/{}.png", id);
    Ok(Json(json!({
        "id": id,
        "url": url,
        "symbol": payload.symbol,
        "timeframe": payload.timeframe
    })))
}

pub async fn get_snapshot(Path(id): Path<String>) -> Result<Response, (StatusCode, &'static str)> {
    let clean_id = id.trim_end_matches(".png");
    if !clean_id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err((StatusCode::BAD_REQUEST, "invalid snapshot ID"));
    }

    let file_path = format!("/tmp/pia_snapshots/{}.png", clean_id);
    match std::fs::read(&file_path) {
        Ok(bytes) => Ok(Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "image/png")
            .header(header::CACHE_CONTROL, "public, max-age=86400")
            .body(Body::from(bytes))
            .unwrap()),
        Err(_) => Err((StatusCode::NOT_FOUND, "snapshot not found")),
    }
}
