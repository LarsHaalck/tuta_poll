use crate::http_client::{HttpClient, Method};
use crate::serialize::*;
use anyhow::Result;
use serde::Deserialize;
use tracing::{debug, trace};

#[derive(Deserialize)]
struct Response {
    #[serde(with = "serde_format")]
    _format: (),
    mailbox: String,
}

pub async fn fetch(client: &HttpClient, group: &str) -> Result<String> {
    debug!("Fetching mailboxgrouproot");
    let url = url::Url::parse(super::BASE_URL)?
        .join(format!("/rest/tutanota/mailboxgrouproot/{}", group).as_str())?;

    let mailbox = client
        .send(Method::AuthGet, url, None)
        .await?
        .json::<Response>()
        .await?
        .mailbox;

    debug!("Fetched mailboxgrouproot");
    trace!("mailboxgrouproot: {:?}", mailbox);
    Ok(mailbox)
}
