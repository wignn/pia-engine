use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, Method, StatusCode, Uri},
    response::Response,
};

use crate::state::AppState;

pub async fn proxy_request(
    State(state): State<AppState>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Body,
) -> Response {
    let path = uri.path();
    let target_base = if path.starts_with("/api/v1/market/why") || path == "/api/v1/analyze" {
        &state.config.intelligence_service_url
    } else if path.starts_with("/api/v1/market/") {
        &state.config.market_data_url
    } else if path.starts_with("/api/v1/forex/") || path.starts_with("/api/v1/stock/") {
        &state.config.news_service_url
    } else {
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

pub fn text_response(status: StatusCode, body: &'static str) -> Response {
    Response::builder()
        .status(status)
        .body(Body::from(body))
        .unwrap()
}
