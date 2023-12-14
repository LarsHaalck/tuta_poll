use crate::types::Folder;
use anyhow::Result;
use tracing::{debug, trace};

pub async fn fetch(access_token: &str, folders: &str) -> Result<Vec<Folder>> {
    debug!("Fetching mailfolder");
    let mut url = url::Url::parse(super::BASE_URL)?
        .join(format!("/rest/tutanota/mailfolder/{}", folders).as_str())?;
    url.query_pairs_mut()
        .append_pair("start", "------------")
        .append_pair("count", "1000")
        .append_pair("reverse", "false");

    let folders = crate::request::auth_get(url, access_token)
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<Folder>>()
        .await?;

    debug!("Fetched mailfolder");
    trace!("mailfolder: {:#?}", folders);
    Ok(folders)
}
