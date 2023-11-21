use super::serialize::*;
use anyhow::Result;
use serde::Deserialize;

// export enum GroupType {
// 	User = "0",
// 	Admin = "1",
// 	MailingList = "2",
// 	Customer = "3",
// 	External = "4",
// 	Mail = "5",
// 	Contact = "6",
// 	File = "7",
// 	LocalAdmin = "8",
// 	Calendar = "9",
// 	Template = "10",
// 	ContactList = "11",
// }

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Membership {
    pub group: String,
    pub group_type: String,
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
        .json::<User>()?;
    Ok(response)
}
