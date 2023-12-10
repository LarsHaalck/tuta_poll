use anyhow::Result;
use crate::types::User;
use tracing::debug;


pub fn fetch(access_token: &str, user: &str) -> Result<User> {
    debug!("Fetching user");

    let url =
        url::Url::parse(super::BASE_URL)?.join(format!("/rest/sys/user/{}", user).as_str())?;

    let user = crate::request::auth_get(url, access_token)
        .send()?
        .error_for_status()?
        .json::<User>()?;

    debug!("Fetched user: {:#?}", user);
    Ok(user)
}
