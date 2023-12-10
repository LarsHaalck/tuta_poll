use crate::serialize::*;
use anyhow::Result;
use serde::Deserialize;
use tracing::debug;

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
    debug!("Fetching mailbox");
    let url = url::Url::parse(super::BASE_URL)?
        .join(format!("/rest/tutanota/mailbox/{}", mailbox).as_str())?;

    let response = crate::request::auth_get(url, access_token)
        .send()?
        .error_for_status()?
        .json::<Response>()?;

    debug!("Fetched mailbox: {:?}", response.folders.folders);
    Ok(response.folders.folders)
}