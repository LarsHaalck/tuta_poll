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

pub fn fetch(access_token: &str, group: &str) -> Result<String> {
    debug!("Fetching mailboxgrouproot");
    let url = url::Url::parse(super::BASE_URL)?
        .join(format!("/rest/tutanota/mailboxgrouproot/{}", group).as_str())?;

    let mailbox = crate::request::auth_get(url, access_token)
        .send()?
        .error_for_status()?
        .json::<Response>()?
        .mailbox;

    debug!("Fetched mailboxgrouproot");
    trace!("mailboxgrouproot: {:?}", mailbox);
    Ok(mailbox)
}
