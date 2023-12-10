use crate::types::{Id, BucketPermission};
use anyhow::Result;
use tracing::debug;

pub fn fetch(access_token: &str, bucket: &Id) -> Result<Vec<BucketPermission>> {
    debug!("Fetching bucket permission");
    let mut url = url::Url::parse(super::BASE_URL)?
        .join(format!("/rest/sys/bucketpermission/{}", bucket).as_str())?;
    url.query_pairs_mut()
        .append_pair("start", "------------")
        .append_pair("count", "1000")
        .append_pair("reverse", "false");

    let response = crate::request::auth_get(url, access_token)
        .send()?
        .error_for_status()?
        .json::<Vec<BucketPermission>>()?;

    debug!("Fetched bucket permission: {:#?}", response);
    Ok(response)
}
