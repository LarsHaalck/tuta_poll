use crate::http_client::{HttpClient, Method};
use crate::serialize::*;
use crate::types::Aes128Key;
use anyhow::{Context, Result};
use serde::Deserialize;
use tracing::debug;

#[derive(Deserialize, Debug)]
struct Response {
    #[serde(with = "serde_format")]
    _format: (),
    #[serde(with = "serde_base64_16")]
    salt: Aes128Key,
}

pub async fn fetch(client: &HttpClient, email_address: &str) -> Result<Aes128Key> {
    debug!("Fetching salt");

    let payload = format!(
        "_body={}",
        serde_json::json!({
            "_format": "0",
            "mailAddress": email_address
        })
    )
    .to_string();

    let mut url = url::Url::parse(super::BASE_URL)?.join("/rest/sys/saltservice")?;
    url.set_query(Some(&payload));

    let response = client
        .send(Method::Get, url, None)
        .await?
        .json::<Response>()
        .await?;

    debug!("Fetched salt");
    response.salt.try_into().context("salt is too big")
}
