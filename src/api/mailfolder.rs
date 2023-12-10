use crate::types::Folder;
use anyhow::Result;
use tracing::debug;

pub fn fetch(access_token: &str, folders: &str) -> Result<Vec<Folder>> {
    debug!("Fetching mailfolder");
    let mut url = url::Url::parse(super::BASE_URL)?
        .join(format!("/rest/tutanota/mailfolder/{}", folders).as_str())?;
    url.query_pairs_mut()
        .append_pair("start", "------------")
        .append_pair("count", "1000")
        .append_pair("reverse", "false");

    let response = crate::request::auth_get(url, access_token)
        .send()?
        .error_for_status()?
        .json::<Vec<Folder>>()?;

    debug!("Fetched mailfolder: {:#?}", response);
    Ok(response)
}
