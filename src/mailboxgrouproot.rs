use serde::Deserialize;
use super::serialize::*;
use anyhow::Result;

#[derive(Deserialize)]
struct Response {
    #[serde(with = "serde_format")]
    _format: (),
    mailbox: String,
}

pub fn fetch(access_token: &str, group: &str)-> Result<String> {
    let url =
        url::Url::parse(super::BASE_URL)?.join(format!("/rest/tutanota/mailboxgrouproot/{}", group).as_str())?;

    let response = super::request::auth_get(url, access_token)
        .send()?
        .json::<Response>()?;
    Ok(response.mailbox)
}
