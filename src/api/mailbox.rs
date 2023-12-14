use crate::serialize::*;
use anyhow::Result;
use serde::Deserialize;
use tracing::{debug, trace};

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

pub async fn fetch(access_token: &str, mailbox: &str) -> Result<String> {
    debug!("Fetching mailbox");
    let url = url::Url::parse(super::BASE_URL)?
        .join(format!("/rest/tutanota/mailbox/{}", mailbox).as_str())?;

    let folders = crate::request::auth_get(url, access_token)
        .send()
        .await?
        .error_for_status()?
        .json::<Response>()
        .await?
        .folders
        .folders;

    debug!("Fetched mailbox");
    trace!("mailbox: {:#?}", folders);
    Ok(folders)
}
