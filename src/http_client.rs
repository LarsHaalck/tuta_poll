use anyhow::{bail, Context, Result};
use reqwest::{header::HeaderMap, Client, Response, StatusCode};
use std::time::Duration;
use tracing::warn;
use url::Url;

#[derive(Clone)]
pub enum Method {
    Get,
    Post,
    AuthGet,
    AuthPut,
}

pub struct HttpClient {
    client: Client,
    access_token: Option<String>,
}

impl HttpClient {
    pub fn new() -> HttpClient {
        let client = reqwest::Client::new();
        HttpClient {
            client,
            access_token: None,
        }
    }

    pub fn set_access_token(&mut self, access_token: String) {
        self.access_token = Some(access_token);
    }
    pub fn get_access_token(&self) -> Option<&String> {
        self.access_token.as_ref()
    }

    fn get_access_token_header(&self) -> Result<HeaderMap> {
        let mut request_headers = HeaderMap::new();
        request_headers.insert(
            "accessToken",
            self.access_token
                .clone()
                .context("Client needs to be authenticated first")?
                .parse()?,
        );
        Ok(request_headers)
    }

    pub async fn send(&self, method: Method, url: Url, body: Option<String>) -> Result<Response> {
        let request_method: reqwest::Method;
        let mut request_headers = HeaderMap::new();
        match method {
            Method::Get => {
                request_method = reqwest::Method::GET;
            }
            Method::Post => {
                request_method = reqwest::Method::POST;
            }
            Method::AuthGet => {
                request_method = reqwest::Method::GET;
                request_headers = self.get_access_token_header()?;
            }
            Method::AuthPut => {
                request_method = reqwest::Method::PUT;
                request_headers = self.get_access_token_header()?;
            }
        };

        loop {
            let mut curr_request = self
                .client
                .request(request_method.clone(), url.clone())
                .headers(request_headers.clone());
            if let Some(ref b) = body {
                curr_request = curr_request.body(b.clone());
            }
            let curr_request = curr_request.build()?;
            let response = self.client.execute(curr_request).await?;
            let headers = response.headers().clone();
            match response.error_for_status() {
                Err(e) => {
                    if e.status() == Some(StatusCode::TOO_MANY_REQUESTS) {
                        if let Some(duration) = Self::get_retry_duration(&headers) {
                            warn!("Rate limited, retrying in {} seconds", duration.as_secs());
                            tokio::time::sleep(duration).await;
                        }
                    } else {
                        bail!(e.to_string());
                    }
                }
                Ok(res) => return Ok(res),
            }
        }
    }

    fn get_retry_duration(header_map: &HeaderMap) -> Option<Duration> {
        if let Some(val) = header_map.get("Retry-After") {
            if let Ok(retry_value) = val.to_str() {
                if let Ok(duration) = retry_value.parse::<u64>() {
                    return Some(Duration::from_secs(duration));
                }
            }
        }
        None
    }
}
