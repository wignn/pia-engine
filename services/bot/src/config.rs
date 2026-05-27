use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub token: String,
    pub client_id: String,
    pub realtime_ws_url: String,
    pub api_http_url: String,
    pub db_path: String,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let token = env::var("TOKEN").map_err(|_| "TOKEN not configured in .env")?;
        let client_id = env::var("CLIENT_ID").map_err(|_| "CLIENT_ID not configured in .env")?;
        let realtime_ws_url = env::var("REALTIME_GATEWAY_WS_URL")
            .or_else(|_| env::var("realtime_ws_url"))
            .unwrap_or_else(|_| "ws://localhost:8020".to_string());
        let api_http_url = env::var("API_GATEWAY_URL")
            .or_else(|_| env::var("api_http_url"))
            .unwrap_or_else(|_| {
                realtime_ws_url
                    .replace("wss://", "https://")
                    .replace("ws://", "http://")
            });

        let db_path = env::var("DATABASE_PATH").unwrap_or_else(|_| "bot.db".to_string());

        Ok(Self {
            token,
            client_id,
            realtime_ws_url,
            api_http_url,
            db_path,
        })
    }
}
