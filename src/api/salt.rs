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

pub fn fetch(email_address: &str) -> Result<Aes128Key> {
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

    let response = reqwest::blocking::get(url)?
        .error_for_status()?
        .json::<Response>()?;
    response.salt.try_into().context("failed")
}
