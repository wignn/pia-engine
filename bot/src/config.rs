use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub token: String,
    pub client_id: String,
    /// WebSocket URL for real-time event stream (e.g. ws://localhost:4000)
    pub core_ws_url: String,
    /// HTTP base URL for REST API calls (e.g. http://localhost:4000)
    pub core_http_url: String,
    pub db_path: String,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let token = env::var("TOKEN").map_err(|_| "TOKEN not configured in .env")?;
        let client_id = env::var("CLIENT_ID").map_err(|_| "CLIENT_ID not configured in .env")?;

        let core_ws_url = env::var("CORE_WS_URL")
            .unwrap_or_else(|_| "ws://localhost:4000".to_string());

        // Derive HTTP URL from WS URL if not explicitly set
        let core_http_url = env::var("CORE_HTTP_URL").unwrap_or_else(|_| {
            core_ws_url
                .replace("wss://", "https://")
                .replace("ws://", "http://")
        });

        let db_path = env::var("DATABASE_PATH")
            .unwrap_or_else(|_| "bot.db".to_string());

        Ok(Self {
            token,
            client_id,
            core_ws_url,
            core_http_url,
            db_path,
        })
    }
}
