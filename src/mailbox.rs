use super::serialize::*;
use serde::Deserialize;
use anyhow::Result;

#[derive(Deserialize)]
struct Response {
    #[serde(with = "serde_format")]
    _format: (),
    folders: Folders,
}

#[derive(Deserialize)]
struct Folders {
    folders: String,
}

pub fn fetch(access_token: &str, mailbox: &str) -> Result<String> {
    let url = url::Url::parse(super::BASE_URL)?
        .join(format!("/rest/tutanota/mailbox/{}", mailbox).as_str())?;

    let response = super::request::auth_get(url, access_token)
        .send()?
        .error_for_status()?
        .json::<Response>()?;
    Ok(response.folders.folders)
}
