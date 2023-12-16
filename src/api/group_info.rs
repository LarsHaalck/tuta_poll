use crate::http_client::{HttpClient, Method};
use crate::types::{GroupInfo, IdTuple};
use anyhow::Result;
use tracing::{debug, trace};

pub async fn fetch(client: &HttpClient, group: &IdTuple) -> Result<GroupInfo> {
    debug!("Fetching groupinfo");
    let url = url::Url::parse(super::BASE_URL)?
        .join(format!("/rest/sys/groupinfo/{}/{}", group.0, group.1).as_str())?;

    let group_info = client
        .send(Method::AuthGet, url, None)
        .await?
        .json::<GroupInfo>()
        .await?;

    debug!("Fetched groupinfo");
    trace!("groupinfo: {:#?}", group_info);

    Ok(group_info)
}
