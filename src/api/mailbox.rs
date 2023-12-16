use crate::http_client::{HttpClient, Method};
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

pub async fn fetch(client: &HttpClient, mailbox: &str) -> Result<String> {
    debug!("Fetching mailbox");
    let url = url::Url::parse(super::BASE_URL)?
        .join(format!("/rest/tutanota/mailbox/{}", mailbox).as_str())?;

    let folders = client
        .send(Method::AuthGet, url, None)
        .await?
        .json::<Response>()
        .await?
        .folders
        .folders;

    debug!("Fetched mailbox");
    trace!("mailbox: {:#?}", folders);
    Ok(folders)
}
