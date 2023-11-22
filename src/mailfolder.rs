use super::serialize::*;
use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub enum MailFolderType {
    Custom,
    Inbox,
    Sent,
    Trash,
    Archive,
    Spam,
    Draft,
}

#[derive(Debug, Deserialize)]
pub struct Folder {
    #[serde(with = "serde_format")]
    _format: (),
    #[serde(with = "serde_mail_folder_type", rename = "folderType")]
    pub folder_type: MailFolderType,
    #[serde(rename = "_id")]
    pub id: (String, String),
    pub mails: String,
    #[serde(with = "serde_base64")]
    pub name: Vec<u8>,
    #[serde(with = "serde_base64", rename = "_ownerEncSessionKey")]
    pub owner_enc_session_key: Vec<u8>,
}

pub fn fetch(access_token: &str, folders: &str) -> Result<Vec<Folder>> {
    let mut url = url::Url::parse(super::BASE_URL)?
        .join(format!("/rest/tutanota/mailfolder/{}", folders).as_str())?;
    url.query_pairs_mut()
        .append_pair("start", "------------")
        .append_pair("count", "1000")
        .append_pair("reverse", "false");

    let response = super::request::auth_get(url, access_token)
        .send()?
        .json::<Vec<Folder>>()?;
    Ok(response)
}
