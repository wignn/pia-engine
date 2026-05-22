use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use tracing::warn;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JwtClaims {
    pub sub: String,
    pub email: String,
    pub plan: String,
    pub exp: usize,
    pub iat: usize,
}

pub trait JwtUser {
    fn jwt_sub(&self) -> String;
    fn jwt_email(&self) -> &str;
    fn jwt_plan(&self) -> &str;
}

pub fn create_jwt_for_user<U: JwtUser>(
    user: &U,
    secret: &str,
    expiry_days: u64,
) -> Result<String, jsonwebtoken::errors::Error> {
    create_jwt(
        user.jwt_sub(),
        user.jwt_email(),
        user.jwt_plan(),
        secret,
        expiry_days,
    )
}

pub fn create_jwt(
    sub: String,
    email: &str,
    plan: &str,
    secret: &str,
    expiry_days: u64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let exp = now + Duration::days(expiry_days as i64);

    let claims = JwtClaims {
        sub,
        email: email.to_string(),
        plan: plan.to_string(),
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| {
        warn!(error = %e, "failed to create JWT");
        e
    })
}

pub fn decode_jwt(token: &str, secret: &str) -> Option<JwtClaims> {
    decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .ok()
    .map(|data| data.claims)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OAuthStateClaims {
    pub provider: String,
    pub exp: usize,
    pub iat: usize,
}

pub fn create_oauth_state(
    provider: &str,
    secret: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let claims = OAuthStateClaims {
        provider: provider.to_string(),
        exp: (now + Duration::minutes(10)).timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| {
        warn!(error = %e, "failed to create OAuth state");
        e
    })
}

pub fn validate_oauth_state(provider: &str, state_token: &str, secret: &str) -> bool {
    decode::<OAuthStateClaims>(
        state_token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims.provider == provider)
    .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::{create_oauth_state, validate_oauth_state};

    #[test]
    fn oauth_state_is_signed_and_provider_bound() {
        let secret = "test-secret-with-enough-entropy";
        let state = create_oauth_state("github", secret).unwrap();

        assert!(validate_oauth_state("github", &state, secret));
        assert!(!validate_oauth_state("google", &state, secret));
        assert!(!validate_oauth_state("github", &state, "different-secret"));
    }
}
