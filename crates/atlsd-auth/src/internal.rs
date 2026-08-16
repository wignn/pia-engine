use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use sha2::{Digest, Sha256};

pub const INTERNAL_API_KEY_HEADER: &str = "x-internal-api-key";

pub fn secrets_match(expected: &str, provided: &str) -> bool {
    let expected_digest = Sha256::digest(expected.as_bytes());
    let provided_digest = Sha256::digest(provided.as_bytes());
    expected_digest
        .iter()
        .zip(provided_digest.iter())
        .fold(0u8, |acc, (a, b)| acc | (a ^ b))
        == 0
}

#[derive(Clone, Default)]
pub struct InternalAuth {
    key: Option<String>,
}

impl InternalAuth {
    pub fn from_env() -> Self {
        Self::new(std::env::var("INTERNAL_API_KEY").ok())
    }

    pub fn new(key: Option<String>) -> Self {
        let key = key.filter(|key| !key.trim().is_empty());
        if key.is_none() {
            tracing::warn!(
                "INTERNAL_API_KEY is empty; internal endpoints are UNAUTHENTICATED (dev mode)"
            );
        }
        Self { key }
    }

    pub fn enabled(&self) -> bool {
        self.key.is_some()
    }

    pub fn check(&self, provided: Option<&str>) -> bool {
        match (&self.key, provided) {
            (Some(expected), Some(provided)) => secrets_match(expected, provided),
            _ => false,
        }
    }
}

const PUBLIC_PATHS: &[&str] = &["/health", "/metrics"];

pub async fn require_internal_key(
    State(auth): State<InternalAuth>,
    request: Request,
    next: Next,
) -> Response {
    if !auth.enabled() || PUBLIC_PATHS.contains(&request.uri().path()) {
        return next.run(request).await;
    }

    let provided = request
        .headers()
        .get(INTERNAL_API_KEY_HEADER)
        .and_then(|value| value.to_str().ok());

    if auth.check(provided) {
        next.run(request).await
    } else {
        tracing::warn!(
            path = %request.uri().path(),
            "rejected unauthenticated internal request"
        );
        StatusCode::UNAUTHORIZED.into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secrets_match_compares_digests() {
        assert!(secrets_match("secret", "secret"));
        assert!(!secrets_match("secret", "Secret"));
        assert!(!secrets_match("secret", ""));
        assert!(!secrets_match("", "secret"));
        assert!(secrets_match("", ""));
    }

    #[test]
    fn disabled_auth_accepts_everything() {
        let auth = InternalAuth::new(None);
        assert!(!auth.enabled());
        assert!(!auth.check(Some("anything")));
    }

    #[test]
    fn enabled_auth_requires_exact_key() {
        let auth = InternalAuth::new(Some("internal-key".to_string()));
        assert!(auth.enabled());
        assert!(auth.check(Some("internal-key")));
        assert!(!auth.check(Some("wrong")));
        assert!(!auth.check(None));
    }

    #[test]
    fn blank_key_is_treated_as_disabled() {
        let auth = InternalAuth::new(Some("   ".to_string()));
        assert!(!auth.enabled());
    }
}
