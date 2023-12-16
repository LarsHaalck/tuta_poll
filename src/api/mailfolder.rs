use crate::http_client::{HttpClient, Method};
use crate::types::Folder;
use anyhow::Result;
use tracing::{debug, trace};

pub async fn fetch(client: &HttpClient, folders: &str) -> Result<Vec<Folder>> {
    debug!("Fetching mailfolder");
    let mut url = url::Url::parse(super::BASE_URL)?
        .join(format!("/rest/tutanota/mailfolder/{}", folders).as_str())?;
    url.query_pairs_mut()
        .append_pair("start", "------------")
        .append_pair("count", "1000")
        .append_pair("reverse", "false");

    let folders = client
        .send(Method::AuthGet, url, None)
        .await?
        .json::<Vec<Folder>>()
        .await?;

    debug!("Fetched mailfolder");
    trace!("mailfolder: {:#?}", folders);
    Ok(folders)
}
