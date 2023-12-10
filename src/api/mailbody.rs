use crate::serialize::*;
use anyhow::Result;
use serde::Deserialize;
use tracing::debug;
use crate::types::Base64;

#[derive(Debug, Deserialize)]
pub struct Mailbody {
    #[serde(with = "serde_format")]
    _format: (),
    #[serde(with = "serde_base64", rename = "compressedText")]
    pub text: Base64,
}

pub fn fetch(access_token: &str, body: &str) -> Result<Vec<u8>> {
    debug!("Fetching body");
    let url = url::Url::parse(super::BASE_URL)?
        .join(format!("/rest/tutanota/mailbody/{}", body).as_str())?;
    let response = crate::request::auth_get(url, access_token)
        .send()?
        .error_for_status()?
        .json::<Mailbody>()?;

    debug!("Fetched body");
    Ok(response.text)
}
