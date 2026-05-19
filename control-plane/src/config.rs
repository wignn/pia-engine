use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub redis_channel_prefix: String,
    pub port: u16,
    pub admin_api_key: String,
    pub log_level: String,
    // JWT
    pub jwt_secret: String,
    pub jwt_expiry_days: u64,
    // OAuth
    pub google_client_id: String,
    pub google_client_secret: String,
    pub github_client_id: String,
    pub github_client_secret: String,
    pub frontend_url: String,
    // Encryption
    pub encryption_key: String,
}

impl Config {
    pub fn load() -> Self {
        Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/forex".into()),
            redis_url: env::var("REDIS_URL").unwrap_or_default(),
            redis_channel_prefix: env::var("REDIS_CHANNEL_PREFIX")
                .unwrap_or_else(|_| "world-info".into()),
            port: env::var("PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(8080),
            admin_api_key: env::var("ADMIN_API_KEY").unwrap_or_default(),
            log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".into()),
            // JWT
            jwt_secret: load_jwt_secret(),
            jwt_expiry_days: env::var("JWT_EXPIRY_DAYS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(7),
            // OAuth
            google_client_id: env::var("GOOGLE_CLIENT_ID").unwrap_or_default(),
            google_client_secret: env::var("GOOGLE_CLIENT_SECRET").unwrap_or_default(),
            github_client_id: env::var("GITHUB_CLIENT_ID").unwrap_or_default(),
            github_client_secret: env::var("GITHUB_CLIENT_SECRET").unwrap_or_default(),
            frontend_url: env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:5173".into()),
            encryption_key: env::var("ENCRYPTION_KEY")
                .unwrap_or_else(|_| load_jwt_secret()),
        }
    }

    pub fn has_google_oauth(&self) -> bool {
        !self.google_client_id.is_empty() && !self.google_client_secret.is_empty()
    }

    pub fn has_github_oauth(&self) -> bool {
        !self.github_client_id.is_empty() && !self.github_client_secret.is_empty()
    }
}

fn load_jwt_secret() -> String {
    match env::var("JWT_SECRET") {
        Ok(value) if !value.trim().is_empty() && value != "change-me-in-production-please" => value,
        _ if cfg!(debug_assertions) => "dev-only-insecure-jwt-secret".into(),
        _ => panic!("JWT_SECRET must be set to a non-default value"),
    }
}
