use std::env;
use url::Url;

pub fn get_env(key: &str, fallback: &str) -> String {
    env::var(key).unwrap_or_else(|_| fallback.to_string())
}

pub fn get_env_u64(key: &str, fallback: u64) -> u64 {
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(fallback)
}

pub fn get_env_f64(key: &str, fallback: f64) -> f64 {
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(fallback)
}

pub fn get_env_any(keys: &[&str], fallback: &str) -> String {
    for key in keys {
        if let Ok(value) = env::var(key) {
            if !value.trim().is_empty() {
                return value;
            }
        }
    }
    fallback.to_string()
}

pub fn sanitize_database_url(input: &str) -> String {
    let mut url = match Url::parse(input) {
        Ok(u) => u,
        Err(_) => return input.to_string(),
    };

    let pairs: Vec<(String, String)> = url
        .query_pairs()
        .filter(|(k, _)| k != "channel_binding")
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    url.query_pairs_mut().clear();
    if !pairs.is_empty() {
        url.query_pairs_mut()
            .extend_pairs(pairs.iter().map(|(k, v)| (k.as_str(), v.as_str())));
    }

    url.to_string()
}
