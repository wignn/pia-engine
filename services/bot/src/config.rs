use std::env;

#[derive(Clone)]
pub struct Config {
    pub token: String,
    pub client_id: String,
    pub api_key: String,
    pub realtime_ws_url: String,
    pub api_http_url: String,
    pub db_path: String,
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("token", &"<redacted>")
            .field("client_id", &self.client_id)
            .field("api_key", &"<redacted>")
            .field("realtime_ws_url", &self.realtime_ws_url)
            .field("api_http_url", &self.api_http_url)
            .field("db_path", &self.db_path)
            .finish()
    }
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let token = env::var("TOKEN").map_err(|_| "TOKEN not configured in .env")?;
        let client_id = env::var("CLIENT_ID").map_err(|_| "CLIENT_ID not configured in .env")?;
        let api_key = env::var("API_KEY").map_err(|_| "API_KEY not configured in .env")?;
        let realtime_ws_url = normalize_ws_url(
            env::var("REALTIME_GATEWAY_WS_URL")
                .or_else(|_| env::var("realtime_ws_url"))
                .unwrap_or_else(|_| "ws://localhost:8020/ws/v1".to_string()),
        );
        let api_http_url = normalize_http_url(
            env::var("API_GATEWAY_URL")
                .or_else(|_| env::var("api_http_url"))
                .unwrap_or_else(|_| "http://localhost:8000".to_string()),
        );

        let db_path = env::var("DATABASE_PATH").unwrap_or_else(|_| "bot.db".to_string());

        Ok(Self {
            token,
            client_id,
            api_key,
            realtime_ws_url,
            api_http_url,
            db_path,
        })
    }
}

fn normalize_http_url(url: String) -> String {
    url.trim_end_matches('/')
        .trim_end_matches("/api/v1")
        .trim_end_matches('/')
        .to_string()
}

fn normalize_ws_url(url: String) -> String {
    let url = url.trim_end_matches('/');
    if url.ends_with("/ws/v1") {
        url.to_string()
    } else {
        format!("{url}/ws/v1")
    }
}

#[cfg(test)]
mod tests {
    use super::{normalize_http_url, normalize_ws_url};

    #[test]
    fn normalize_gateway_urls() {
        assert_eq!(
            normalize_http_url("https://gateway/api/v1/".into()),
            "https://gateway"
        );
        assert_eq!(
            normalize_ws_url("wss://realtime".into()),
            "wss://realtime/ws/v1"
        );
    }
}
