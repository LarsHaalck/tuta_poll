use anyhow::Result;
use serde::Deserialize;
use tracing::debug;
use super::serialize::*;
use anyhow::Error;

#[derive(Deserialize, Debug)]
struct Response {
    #[serde(with = "serde_format")]
    _format: (),
    #[serde(with = "serde_base64")]
    salt: Vec<u8>,
}

pub fn fetch(email_address: &str) -> Result<Vec<u8>> {
    let payload = format!("_body={}", serde_json::json!({
        "_format": "0",
        "mailAddress": email_address
    })).to_string();

    let mut url = url::Url::parse(super::BASE_URL)?
        .join("/rest/sys/saltservice")?;
    url.set_query(Some(&payload));

    debug!("request url: {:?}", url.as_str());
    let response = reqwest::blocking::get(url)?.json::<Response>()?;
    debug!("response: {:?}", response);

    let salt = response.salt;
    if salt.len() == 16 {
        Ok(salt)
    } else {
        Err(Error::msg("Salt has wrong length"))
    }
}
