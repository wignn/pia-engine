use axum::{extract::Request, http::header};

pub fn extract_key(request: &Request) -> Option<String> {
    extract_rest_key(request)
}

pub fn extract_rest_key(request: &Request) -> Option<String> {
    if let Some(val) = request.headers().get("X-API-Key") {
        if let Ok(s) = val.to_str() {
            if !s.is_empty() {
                return Some(s.to_string());
            }
        }
    }

    extract_bearer(request)
}

pub fn extract_ws_key(request: &Request) -> Option<String> {
    extract_query_key(request)
}

fn extract_query_key(request: &Request) -> Option<String> {
    request.uri().query().and_then(|q| {
        url::form_urlencoded::parse(q.as_bytes())
            .find(|(k, _)| k == "api_key" || k == "token")
            .map(|(_, v)| v.to_string())
    })
}

pub fn extract_bearer(request: &Request) -> Option<String> {
    if let Some(val) = request.headers().get(header::AUTHORIZATION) {
        if let Ok(s) = val.to_str() {
            if let Some(token) = s.strip_prefix("Bearer ") {
                if !token.is_empty() {
                    return Some(token.to_string());
                }
            }
        }
    }

    if let Some(val) = request.headers().get(header::COOKIE) {
        if let Ok(s) = val.to_str() {
            for cookie in s.split(';') {
                let cookie = cookie.trim();
                if let Some(token) = cookie.strip_prefix("wi_jwt=") {
                    if !token.is_empty() {
                        return Some(token.to_string());
                    }
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;

    fn request(uri: &str) -> Request {
        Request::builder().uri(uri).body(Body::empty()).unwrap()
    }

    #[test]
    fn rest_prefers_api_key_header_before_bearer() {
        let req = Request::builder()
            .uri("/api/v1/content/scrape?api_key=query-key")
            .header("X-API-Key", "header-key")
            .header(header::AUTHORIZATION, "Bearer bearer-token")
            .body(Body::empty())
            .unwrap();

        assert_eq!(extract_rest_key(&req).as_deref(), Some("header-key"));
    }

    #[test]
    fn rest_ignores_query_credentials() {
        assert_eq!(
            extract_rest_key(&request("/api/v1/content/scrape?api_key=query-key")),
            None
        );
    }

    #[test]
    fn rest_extracts_bearer_token_for_api_key() {
        let req = Request::builder()
            .uri("/api/v1/content/scrape")
            .header(header::AUTHORIZATION, "Bearer bearer-token")
            .body(Body::empty())
            .unwrap();

        assert_eq!(extract_rest_key(&req).as_deref(), Some("bearer-token"));
        assert_eq!(extract_key(&req).as_deref(), Some("bearer-token"));
        assert_eq!(extract_bearer(&req).as_deref(), Some("bearer-token"));
    }

    #[test]
    fn ws_extracts_query_credentials() {
        assert_eq!(
            extract_ws_key(&request("/ws?api_key=query-key")).as_deref(),
            Some("query-key")
        );
        assert_eq!(
            extract_ws_key(&request("/ws?token=query-token")).as_deref(),
            Some("query-token")
        );
    }

    #[test]
    fn ws_ignores_bearer_and_cookie_credentials() {
        let req = Request::builder()
            .uri("/ws?api_key=query-key")
            .header(header::AUTHORIZATION, "Bearer bearer-token")
            .header(header::COOKIE, "wi_jwt=cookie-token")
            .body(Body::empty())
            .unwrap();

        assert_eq!(extract_ws_key(&req).as_deref(), Some("query-key"));
    }

    #[test]
    fn cookie_jwt_is_bearer_only() {
        let req = Request::builder()
            .uri("/dashboard")
            .header(header::COOKIE, "theme=dark; wi_jwt=cookie-token; other=1")
            .body(Body::empty())
            .unwrap();

        assert_eq!(extract_bearer(&req).as_deref(), Some("cookie-token"));
        assert_eq!(extract_rest_key(&req).as_deref(), Some("cookie-token"));
        assert_eq!(extract_ws_key(&req), None);
    }
}
