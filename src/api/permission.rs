use crate::http_client::{HttpClient, Method};
use crate::types::{Id, Permission};
use anyhow::Result;
use tracing::{debug, trace};

pub async fn fetch(client: &HttpClient, permission: &Id) -> Result<Vec<Permission>> {
    debug!("Fetching permission");
    let mut url = url::Url::parse(super::BASE_URL)?
        .join(format!("/rest/sys/permission/{}", permission).as_str())?;
    url.query_pairs_mut()
        .append_pair("start", "------------")
        .append_pair("count", "1000")
        .append_pair("reverse", "false");

    let permission = client
        .send(Method::AuthGet, url, None)
        .await?
        .json::<Vec<Permission>>()
        .await?;

    debug!("Fetched permission");
    trace!("permission: {:#?}", permission);
    Ok(permission)
}
