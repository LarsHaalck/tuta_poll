use super::serialize::*;
use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Mailbody {
    #[serde(with = "serde_format")]
    _format: (),
    #[serde(with = "serde_base64", rename = "compressedText")]
    pub text: Vec<u8>,
}

pub fn fetch(access_token: &str, body: &str) -> Result<Vec<u8>> {
    let url = url::Url::parse(super::BASE_URL)?
        .join(format!("/rest/tutanota/mailbody/{}", body).as_str())?;
    let response = super::request::auth_get(url, access_token)
        .send()?
        .json::<Mailbody>()?;
    Ok(response.text)
}
