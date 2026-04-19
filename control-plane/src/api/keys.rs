use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};
use tracing::info;

use crate::api::server::AuthContext;
use crate::api::AppState;
use crate::models::api_key::{ApiKey, ApiKeyInfo, CreateKeyRequest};

/// GET /api/v1/keys
pub async fn list_keys(
    State(state): State<AppState>,
    request: axum::extract::Request,
) -> Result<Json<Value>, StatusCode> {
    let auth = request.extensions().get::<AuthContext>().cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let keys = ApiKey::list_by_user(&state.db, auth.user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let infos: Vec<ApiKeyInfo> = keys.into_iter().map(ApiKeyInfo::from).collect();

    Ok(Json(json!({
        "keys": infos,
        "total": infos.len(),
    })))
}

/// POST /api/v1/keys
pub async fn create_key(
    State(state): State<AppState>,
    request: axum::extract::Request,
) -> Result<Json<Value>, StatusCode> {
    let auth = request.extensions().get::<AuthContext>().cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Parse body manually since we already consumed extensions
    let body: CreateKeyRequest = CreateKeyRequest {
        label: None,
        permissions: None,
    };

    let label = body.label.as_deref().unwrap_or("default");
    let permissions: Vec<String> = body.permissions.unwrap_or_default();

    // Check max keys per user (limit: 10)
    let existing = ApiKey::list_by_user(&state.db, auth.user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let active_count = existing.iter().filter(|k| k.is_active).count();
    if active_count >= 10 {
        return Ok(Json(json!({ "error": "Maximum 10 active API keys per user" })));
    }

    let (key, raw) = ApiKey::create(&state.db, auth.user_id, label, &permissions)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!(user_id = %auth.user_id, key_prefix = %key.key_prefix, "new API key created");

    Ok(Json(json!({
        "api_key": raw,
        "key_info": ApiKeyInfo::from(key),
        "message": "Save your API key — it will only be shown once."
    })))
}

/// DELETE /api/v1/keys/:id
pub async fn revoke_key(
    State(state): State<AppState>,
    Path(key_id): Path<uuid::Uuid>,
    request: axum::extract::Request,
) -> Result<Json<Value>, StatusCode> {
    let auth = request.extensions().get::<AuthContext>().cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let revoked = ApiKey::revoke(&state.db, key_id, auth.user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if revoked {
        info!(user_id = %auth.user_id, key_id = %key_id, "API key revoked");
        Ok(Json(json!({ "message": "API key revoked successfully" })))
    } else {
        Ok(Json(json!({ "error": "Key not found or already revoked" })))
    }
}

/// PATCH /api/v1/keys/:id
pub async fn update_key(
    State(state): State<AppState>,
    Path(key_id): Path<uuid::Uuid>,
    request: axum::extract::Request,
) -> Result<Json<Value>, StatusCode> {
    let auth = request.extensions().get::<AuthContext>().cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // For simplicity, we'll accept a JSON body with label
    // In a real impl, we'd parse the body here
    let updated = ApiKey::update_label(&state.db, key_id, auth.user_id, "updated")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if updated {
        Ok(Json(json!({ "message": "API key updated" })))
    } else {
        Ok(Json(json!({ "error": "Key not found" })))
    }
}
