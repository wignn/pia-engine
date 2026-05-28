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
use crate::sync;

#[derive(serde::Deserialize)]
pub struct UpdateKeyRequest {
    pub label: String,
    pub max_ws_connections: Option<i32>,
}

/// GET /api/v1/keys
pub async fn list_keys(
    State(state): State<AppState>,
    request: axum::extract::Request,
) -> Result<Json<Value>, StatusCode> {
    let auth = request
        .extensions()
        .get::<AuthContext>()
        .cloned()
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
    let auth = request
        .extensions()
        .get::<AuthContext>()
        .cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let body_bytes = axum::body::to_bytes(request.into_body(), 16 * 1024)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let body: CreateKeyRequest = if body_bytes.is_empty() {
        CreateKeyRequest {
            label: None,
            permissions: None,
        }
    } else {
        serde_json::from_slice(&body_bytes).map_err(|_| StatusCode::BAD_REQUEST)?
    };

    let label = normalize_label(body.label.as_deref(), true)?;
    let permissions: Vec<String> = body.permissions.unwrap_or_default();

    let existing = ApiKey::list_by_user(&state.db, auth.user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let active_count = existing.iter().filter(|k| k.is_active).count();
    if active_count >= 10 {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    let (key, raw) = ApiKey::create(&state.db, auth.user_id, &label, &permissions)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!(user_id = %auth.user_id, key_prefix = %key.key_prefix, "new API key created");
    sync::publish_config_changed_for_user(
        &state.redis,
        &state.config.redis_channel_prefix,
        Some(auth.user_id),
    )
    .await;

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
    let auth = request
        .extensions()
        .get::<AuthContext>()
        .cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let revoked = ApiKey::revoke(&state.db, key_id, auth.user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if revoked {
        info!(user_id = %auth.user_id, key_id = %key_id, "API key revoked");
        sync::publish_config_changed_for_user(
            &state.redis,
            &state.config.redis_channel_prefix,
            Some(auth.user_id),
        )
        .await;
        Ok(Json(json!({ "message": "API key revoked successfully" })))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// PATCH /api/v1/keys/:id
pub async fn update_key(
    State(state): State<AppState>,
    Path(key_id): Path<uuid::Uuid>,
    request: axum::extract::Request,
) -> Result<Json<Value>, StatusCode> {
    let auth = request
        .extensions()
        .get::<AuthContext>()
        .cloned()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let body_bytes = axum::body::to_bytes(request.into_body(), 16 * 1024)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let body: UpdateKeyRequest =
        serde_json::from_slice(&body_bytes).map_err(|_| StatusCode::BAD_REQUEST)?;
    let label = normalize_label(Some(&body.label), false)?;
    let max_ws_connections = normalize_ws_limit(body.max_ws_connections)?;

    let label_updated = ApiKey::update_label(&state.db, key_id, auth.user_id, &label)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let limit_updated =
        ApiKey::update_max_ws_connections(&state.db, key_id, auth.user_id, max_ws_connections)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let updated = label_updated || limit_updated;

    if updated {
        sync::publish_config_changed_for_user(
            &state.redis,
            &state.config.redis_channel_prefix,
            Some(auth.user_id),
        )
        .await;
        Ok(Json(json!({ "message": "API key updated" })))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

fn normalize_label(label: Option<&str>, allow_default: bool) -> Result<String, StatusCode> {
    let label = label.unwrap_or("").trim();
    if label.is_empty() {
        return if allow_default {
            Ok("default".to_string())
        } else {
            Err(StatusCode::BAD_REQUEST)
        };
    }
    if label.len() > 80 {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(label.to_string())
}

fn normalize_ws_limit(value: Option<i32>) -> Result<Option<i32>, StatusCode> {
    match value {
        Some(limit) if limit < 1 || limit > 1000 => Err(StatusCode::BAD_REQUEST),
        other => Ok(other),
    }
}

#[cfg(test)]
mod tests {
    use super::{normalize_label, normalize_ws_limit, UpdateKeyRequest};

    #[test]
    fn normalize_label_defaults_only_when_allowed() {
        assert_eq!(normalize_label(None, true).unwrap(), "default");
        assert!(normalize_label(Some(""), false).is_err());
    }

    #[test]
    fn normalize_label_trims_and_limits_length() {
        assert_eq!(
            normalize_label(Some("  trading bot  "), true).unwrap(),
            "trading bot"
        );
        assert!(normalize_label(Some(&"x".repeat(81)), true).is_err());
    }

    #[test]
    fn normalize_ws_limit_accepts_null_or_positive_limit() {
        assert_eq!(normalize_ws_limit(None).unwrap(), None);
        assert_eq!(normalize_ws_limit(Some(2)).unwrap(), Some(2));
        assert!(normalize_ws_limit(Some(0)).is_err());
        assert!(normalize_ws_limit(Some(1001)).is_err());
    }

    #[test]
    fn update_key_request_accepts_ws_limit_payload() {
        let body: UpdateKeyRequest =
            serde_json::from_str(r#"{"label":"browser key","max_ws_connections":2}"#).unwrap();

        assert_eq!(body.label, "browser key");
        assert_eq!(body.max_ws_connections, Some(2));
    }
}
