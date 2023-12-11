use crate::types::{Id, Permission};
use anyhow::Result;
use tracing::{debug, trace};

pub fn fetch(access_token: &str, permission: &Id) -> Result<Vec<Permission>> {
    debug!("Fetching permission");
    let mut url = url::Url::parse(super::BASE_URL)?
        .join(format!("/rest/sys/permission/{}", permission).as_str())?;
    url.query_pairs_mut()
        .append_pair("start", "------------")
        .append_pair("count", "1000")
        .append_pair("reverse", "false");

    let permission = crate::request::auth_get(url, access_token)
        .send()?
        .error_for_status()?
        .json::<Vec<Permission>>()?;

    debug!("Fetched permission");
    trace!("permission: {:#?}", permission);
    Ok(permission)
}
