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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    #[test]
    fn env_helpers_use_values_and_fallbacks() {
        let _guard = env_lock();
        env::remove_var("ATLSD_TEST_STRING");
        env::remove_var("ATLSD_TEST_U64");
        env::remove_var("ATLSD_TEST_F64");

        assert_eq!(get_env("ATLSD_TEST_STRING", "fallback"), "fallback");
        assert_eq!(get_env_u64("ATLSD_TEST_U64", 42), 42);
        assert_eq!(get_env_f64("ATLSD_TEST_F64", 1.5), 1.5);

        env::set_var("ATLSD_TEST_STRING", "value");
        env::set_var("ATLSD_TEST_U64", "99");
        env::set_var("ATLSD_TEST_F64", "2.25");

        assert_eq!(get_env("ATLSD_TEST_STRING", "fallback"), "value");
        assert_eq!(get_env_u64("ATLSD_TEST_U64", 42), 99);
        assert_eq!(get_env_f64("ATLSD_TEST_F64", 1.5), 2.25);

        env::set_var("ATLSD_TEST_U64", "not-a-number");
        env::set_var("ATLSD_TEST_F64", "not-a-number");

        assert_eq!(get_env_u64("ATLSD_TEST_U64", 42), 42);
        assert_eq!(get_env_f64("ATLSD_TEST_F64", 1.5), 1.5);
    }

    #[test]
    fn get_env_any_returns_first_non_empty_value() {
        let _guard = env_lock();
        env::remove_var("ATLSD_TEST_ANY_A");
        env::remove_var("ATLSD_TEST_ANY_B");
        env::set_var("ATLSD_TEST_ANY_A", "   ");
        env::set_var("ATLSD_TEST_ANY_B", "selected");

        assert_eq!(
            get_env_any(&["ATLSD_TEST_ANY_A", "ATLSD_TEST_ANY_B"], "fallback"),
            "selected"
        );
    }

    #[test]
    fn sanitize_database_url_removes_channel_binding_only() {
        assert_eq!(
            sanitize_database_url(
                "postgres://user:pass@localhost/db?sslmode=require&channel_binding=require"
            ),
            "postgres://user:pass@localhost/db?sslmode=require"
        );
        assert_eq!(sanitize_database_url("not a url"), "not a url");
    }
}
