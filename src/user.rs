use super::serialize::*;
use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, PartialEq)]
pub enum GroupType {
	User,
	Admin,
	MailingList,
	Customer,
	External,
	Mail,
	Contact,
	File,
	LocalAdmin,
	Calendar,
	Template,
	ContactList,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Membership {
    pub group: String,
    #[serde(with = "serde_group_type")]
    pub group_type: GroupType,
    #[serde(with = "serde_base64")]
    pub sym_enc_g_key: Vec<u8>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserGroup {
    #[serde(with = "serde_base64")]
    pub sym_enc_g_key: Vec<u8>,
}

#[derive(Debug, Deserialize)]
pub struct User {
    #[serde(with = "serde_format")]
    _format: (),
    pub memberships: Vec<Membership>,
    #[serde(rename = "userGroup")]
    pub user_group: UserGroup,
}

pub fn fetch(access_token: &str, user: &str) -> Result<User> {
    let url =
        url::Url::parse(super::BASE_URL)?.join(format!("/rest/sys/user/{}", user).as_str())?;

    let response = super::request::auth_get(url, access_token)
        .send()?
        .error_for_status()?
        .json::<User>()?;
    Ok(response)
}
