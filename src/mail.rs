use serde::Deserialize;
use super::serialize::*;
use anyhow::Result;

#[derive(Debug, Deserialize)]
pub struct Mail {
    #[serde(with = "serde_format")]
    _format: (),
    pub attachments: Vec<(String, String)>,
    pub body: String,
    #[serde(rename = "_id")]
    pub id: (String, String),
    #[serde(with = "serde_base64", rename = "_ownerEncSessionKey")]
    pub owner_enc_session_key: Vec<u8>,
    #[serde(rename = "receivedDate")]
    pub received_date: String,
    #[serde(rename = "sentDate")]
    pub sent_date: String,
    pub sender: Sender,
    #[serde(with = "serde_base64")]
    pub subject: Vec<u8>,
    pub unread: String,
}

#[derive(Debug, Deserialize)]
pub struct Sender {
    pub address: String,
    #[serde(with = "serde_base64")]
    pub name: Vec<u8>,
}

pub fn fetch(access_token: &str, mails: &str) -> Result<Vec<Mail>> {
    let mut url = url::Url::parse(super::BASE_URL)?
        .join(format!("/rest/tutanota/mail/{}", mails).as_str())?;
    url.query_pairs_mut()
        .append_pair("start", "zzzzzzzzzzzz")
        .append_pair("count", "1000")
        .append_pair("reverse", "true");

    let response = super::request::auth_get(url, access_token)
        .send()?
        .json::<Vec<Mail>>()?;
    Ok(response)
}
