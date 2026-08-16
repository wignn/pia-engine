use std::time::Duration;

use fake_user_agent::get_chrome_rua;
use reqwest::{
    Client,
    header::{ACCEPT, HeaderMap, HeaderValue, USER_AGENT},
};

use crate::error::Result;

#[derive(Clone)]
pub struct HttpClient {
    client: Client,
    max_body_bytes: usize,
}

impl HttpClient {
    pub fn new(timeout: Duration, max_body_bytes: usize) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_str(get_chrome_rua())?);
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("text/html,application/xhtml+xml"),
        );
        Ok(Self {
            client: Client::builder()
                .default_headers(headers)
                .connect_timeout(Duration::from_secs(5))
                .timeout(timeout)
                .pool_max_idle_per_host(8)
                .build()?,
            max_body_bytes,
        })
    }

    pub async fn get_text(&self, url: &str) -> Result<String> {
        let response = self.client.get(url).send().await?.error_for_status()?;
        if response
            .content_length()
            .is_some_and(|n| n > self.max_body_bytes as u64)
        {
            anyhow::bail!("response body exceeds configured limit");
        }
        let body = response.bytes().await?;
        if body.len() > self.max_body_bytes {
            anyhow::bail!("response body exceeds configured limit");
        }
        Ok(String::from_utf8(body.to_vec())?)
    }
}
