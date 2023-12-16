use crate::http_client::{HttpClient, Method};
use crate::types::User;
use anyhow::Result;
use tracing::{debug, trace};

pub async fn fetch(client: &HttpClient, user: &str) -> Result<User> {
    debug!("Fetching user");

    let url =
        url::Url::parse(super::BASE_URL)?.join(format!("/rest/sys/user/{}", user).as_str())?;

    let user = client
        .send(Method::AuthGet, url, None)
        .await?
        .json::<User>()
        .await?;

    debug!("Fetched user");
    trace!("user: {:#?}", user);
    Ok(user)
}
