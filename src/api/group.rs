use crate::http_client::{HttpClient, Method};
use crate::types::{Group, Id};
use anyhow::Result;
use tracing::{debug, trace};

pub async fn fetch(client: &HttpClient, group: &Id) -> Result<Group> {
    debug!("Fetching group");
    let url =
        url::Url::parse(super::BASE_URL)?.join(format!("/rest/sys/group/{}", group).as_str())?;

    let group = client
        .send(Method::AuthGet, url, None)
        .await?
        .json::<Group>()
        .await?;

    debug!("Fetched group");
    trace!("group: {:#?}", group);

    Ok(group)
}
