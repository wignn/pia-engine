use axum::{extract::Request, http::header};

pub fn extract_key(request: &Request) -> Option<String> {
    if let Some(val) = request.headers().get("X-API-Key") {
        if let Ok(s) = val.to_str() {
            if !s.is_empty() {
                return Some(s.to_string());
            }
        }
    }

    if let Some(token) = extract_bearer(request) {
        return Some(token);
    }

    let uri = request.uri();
    uri.query().and_then(|q| {
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
