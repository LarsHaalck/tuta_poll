use anyhow::Result;
use crate::types::{Group, Id};
use tracing::{debug, trace};


pub fn fetch(access_token: &str, group: &Id) -> Result<Group> {
    debug!("Fetching group");
    let url =
        url::Url::parse(super::BASE_URL)?.join(format!("/rest/sys/group/{}", group).as_str())?;

    let group = crate::request::auth_get(url, access_token)
        .send()?
        .error_for_status()?
        .json::<Group>()?;

    debug!("Fetched group");
    trace!("group: {:#?}", group);

    Ok(group)
}