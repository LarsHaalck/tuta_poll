use crate::serialize::*;
use crate::types::Base64;
use anyhow::Result;
use serde::Deserialize;
use tracing::{debug, trace};

#[derive(Debug, Deserialize)]
pub struct Mailbody {
    #[serde(with = "serde_format")]
    _format: (),
    #[serde(with = "serde_base64", rename = "compressedText")]
    pub text: Base64,
}

pub async fn fetch(access_token: &str, body: &str) -> Result<Vec<u8>> {
    debug!("Fetching body");
    let url = url::Url::parse(super::BASE_URL)?
        .join(format!("/rest/tutanota/mailbody/{}", body).as_str())?;

    let text = crate::request::auth_get(url, access_token)
        .send()
        .await?
        .error_for_status()?
        .json::<Mailbody>()
        .await?
        .text;

    debug!("Fetched body");
    trace!("body: {:?}", text);
    Ok(text)
}
