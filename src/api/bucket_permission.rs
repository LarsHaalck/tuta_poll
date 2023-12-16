use crate::http_client::{HttpClient, Method};
use crate::types::{BucketPermission, Id};
use anyhow::Result;
use tracing::{debug, trace};

pub async fn fetch(client: &HttpClient, bucket: &Id) -> Result<Vec<BucketPermission>> {
    debug!("Fetching bucket permission");
    let mut url = url::Url::parse(super::BASE_URL)?
        .join(format!("/rest/sys/bucketpermission/{}", bucket).as_str())?;
    url.query_pairs_mut()
        .append_pair("start", "------------")
        .append_pair("count", "1000")
        .append_pair("reverse", "false");

    let bucket_permission = client
        .send(Method::AuthGet, url, None)
        .await?
        .json::<Vec<BucketPermission>>()
        .await?;

    debug!("Fetched bucket permission");
    trace!("bucket permission: {:#?}", bucket_permission);
    Ok(bucket_permission)
}
