use crate::types::User;
use anyhow::Result;
use tracing::{debug, trace};

pub async fn fetch(access_token: &str, user: &str) -> Result<User> {
    debug!("Fetching user");

    let url =
        url::Url::parse(super::BASE_URL)?.join(format!("/rest/sys/user/{}", user).as_str())?;

    let user = crate::request::auth_get(url, access_token)
        .send()
        .await?
        .error_for_status()?
        .json::<User>()
        .await?;

    debug!("Fetched user");
    trace!("user: {:#?}", user);
    Ok(user)
}
