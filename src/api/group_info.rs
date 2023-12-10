use anyhow::Result;
use crate::types::{GroupInfo, IdTuple};
use tracing::debug;


pub fn fetch(access_token: &str, group: &IdTuple) -> Result<GroupInfo> {
    debug!("Fetching groupinfo");
    let url =
        url::Url::parse(super::BASE_URL)?.join(format!("/rest/sys/groupinfo/{}/{}", group.0, group.1).as_str())?;

    let groupinfo = crate::request::auth_get(url, access_token)
        .send()?
        .error_for_status()?
        .json::<GroupInfo>()?;

    debug!("Fetched groupinfo: {:#?}", groupinfo);

    Ok(groupinfo)
}