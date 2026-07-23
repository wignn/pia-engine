use crate::state::AppState;
use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, Method, StatusCode, Uri},
    response::Response,
};

pub async fn proxy_request(
    State(state): State<AppState>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Body,
) -> Response {
    let path = uri.path();
    let Some(target_base) = target_base_for_path(path, &state.config) else {
        return text_response(StatusCode::NOT_FOUND, "route not found");
    };

    let query = uri
        .query()
        .map(|query| format!("?{query}"))
        .unwrap_or_default();
    let url = format!("{}{}{}", target_base.trim_end_matches('/'), path, query);
    let bytes = match axum::body::to_bytes(body, 2 * 1024 * 1024).await {
        Ok(bytes) => bytes,
        Err(_) => return text_response(StatusCode::BAD_REQUEST, "invalid request body"),
    };

    let mut request = state.http.request(method, &url);
    for (name, value) in headers.iter() {
        if name.as_str().eq_ignore_ascii_case("host") {
            continue;
        }
        request = request.header(name, value);
    }

    match request.body(bytes).send().await {
        Ok(response) => {
            let status = response.status();
            let mut builder = Response::builder().status(status);
            for (name, value) in response.headers() {
                builder = builder.header(name, value);
            }
            match response.bytes().await {
                Ok(bytes) => builder.body(Body::from(bytes)).unwrap(),
                Err(_) => text_response(StatusCode::BAD_GATEWAY, "upstream body error"),
            }
        }
        Err(err) => {
            tracing::warn!(error = %err, url = %url, "gateway upstream request failed");
            text_response(StatusCode::BAD_GATEWAY, "upstream unavailable")
        }
    }
}

fn target_base_for_path<'a>(path: &str, config: &'a crate::config::Config) -> Option<&'a str> {
    if path.starts_with("/api/v1/market/why") || path == "/api/v1/analyze" {
        Some(config.intelligence_service_url.as_str())
    } else if path.starts_with("/api/v1/market/")
        || path.starts_with("/api/v1/rates/")
        || path.starts_with("/api/v1/energy/")
        || path.starts_with("/api/v1/cot/")
        || path.starts_with("/api/v1/fear-greed")
        || path.starts_with("/api/v1/options/")
    {
        Some(config.market_data_url.as_str())
    } else if path.starts_with("/api/v1/forex/")
        || path.starts_with("/api/v1/stock/")
        || path.starts_with("/api/v1/macro/")
        || path.starts_with("/api/v1/admin/forex/")
        || path.starts_with("/api/v1/sec/")
        || path.starts_with("/api/v1/central-banks/")
        || path.starts_with("/api/v1/geosignals")
    {
        Some(config.news_service_url.as_str())
    } else {
        None
    }
}

pub fn text_response(status: StatusCode, body: &'static str) -> Response {
    Response::builder()
        .status(status)
        .body(Body::from(body))
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    fn config() -> Config {
        Config {
            bind_addr: "127.0.0.1:0".to_string(),
            database_url: "postgres://postgres:postgres@localhost/test".to_string(),
            redis_url: String::new(),
            api_keys: vec!["legacy-admin".to_string()],
            admin_api_key: "admin-secret".to_string(),
            log_level: "INFO".to_string(),
            market_data_url: "http://market-data".to_string(),
            news_service_url: "http://news-service".to_string(),
            intelligence_service_url: "http://intelligence-service".to_string(),
        }
    }

    #[test]
    fn macro_dashboard_routes_to_news_service() {
        let cfg = config();

        assert_eq!(
            target_base_for_path("/api/v1/macro/dashboard", &cfg),
            Some(cfg.news_service_url.as_str())
        );
    }

    #[test]
    fn admin_forex_source_routes_to_news_service() {
        let cfg = config();

        assert_eq!(
            target_base_for_path("/api/v1/admin/forex/sources", &cfg),
            Some(cfg.news_service_url.as_str())
        );
        assert_eq!(
            target_base_for_path("/api/v1/admin/forex/sources/feed-fxstreet", &cfg),
            Some(cfg.news_service_url.as_str())
        );
        assert_eq!(
            target_base_for_path("/api/v1/admin/forex/sources/feed-fxstreet/toggle", &cfg),
            Some(cfg.news_service_url.as_str())
        );
        assert_eq!(
            target_base_for_path("/api/v1/admin/forex/sources/test", &cfg),
            Some(cfg.news_service_url.as_str())
        );
    }

    #[test]
    fn routes_new_free_data_pack_paths() {
        let cfg = config();

        assert_eq!(
            target_base_for_path("/api/v1/rates/yield-curve", &cfg),
            Some(cfg.market_data_url.as_str())
        );
        assert_eq!(
            target_base_for_path("/api/v1/sec/filings", &cfg),
            Some(cfg.news_service_url.as_str())
        );
        assert_eq!(
            target_base_for_path("/api/v1/central-banks/latest", &cfg),
            Some(cfg.news_service_url.as_str())
        );
        assert_eq!(
            target_base_for_path("/api/v1/energy/dashboard", &cfg),
            Some(cfg.market_data_url.as_str())
        );
        assert_eq!(
            target_base_for_path("/api/v1/cot/markets", &cfg),
            Some(cfg.market_data_url.as_str())
        );
        assert_eq!(
            target_base_for_path("/api/v1/fear-greed", &cfg),
            Some(cfg.market_data_url.as_str())
        );
        assert_eq!(
            target_base_for_path("/api/v1/geosignals/status", &cfg),
            Some(cfg.news_service_url.as_str())
        );
        assert_eq!(
            target_base_for_path("/api/v1/geosignals", &cfg),
            Some(cfg.news_service_url.as_str())
        );
        assert_eq!(
            target_base_for_path("/api/v1/geosignals/map", &cfg),
            Some(cfg.news_service_url.as_str())
        );
        assert_eq!(
            target_base_for_path("/api/v1/geosignals/assets", &cfg),
            Some(cfg.news_service_url.as_str())
        );
        assert_eq!(
            target_base_for_path("/api/v1/options/summary", &cfg),
            Some(cfg.market_data_url.as_str())
        );
        assert_eq!(
            target_base_for_path("/api/v1/market/trading-halts", &cfg),
            Some(cfg.market_data_url.as_str())
        );
        assert_eq!(
            target_base_for_path("/api/v1/market/corporate-actions", &cfg),
            Some(cfg.market_data_url.as_str())
        );
        assert_eq!(
            target_base_for_path("/api/v1/market/realized-volatility", &cfg),
            Some(cfg.market_data_url.as_str())
        );
        assert_eq!(
            target_base_for_path("/api/v1/market/implied-volatility", &cfg),
            Some(cfg.market_data_url.as_str())
        );
    }
}
