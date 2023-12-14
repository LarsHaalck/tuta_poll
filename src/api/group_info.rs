use crate::types::{GroupInfo, IdTuple};
use anyhow::Result;
use tracing::{debug, trace};

pub async fn fetch(access_token: &str, group: &IdTuple) -> Result<GroupInfo> {
    debug!("Fetching groupinfo");
    let url = url::Url::parse(super::BASE_URL)?
        .join(format!("/rest/sys/groupinfo/{}/{}", group.0, group.1).as_str())?;

    let group_info = crate::request::auth_get(url, access_token)
        .send()
        .await?
        .error_for_status()?
        .json::<GroupInfo>()
        .await?;

    debug!("Fetched groupinfo");
    trace!("groupinfo: {:#?}", group_info);

    Ok(group_info)
}
