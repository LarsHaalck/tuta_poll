use super::serialize::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Mail {
    #[serde(with = "serde_format", rename = "_format")]
    format: (),
    pub auth_status: String,
    pub attachments: Vec<(String, String)>,
    pub bucket_key: (),
    pub body: String,
    pub bcc_recipients: Vec<Sender>,
    pub cc_recipients: Vec<Sender>,
    #[serde(with = "serde_base64")]
    pub confidential: Vec<u8>,
    pub conversation_entry: (String, String),
    pub different_envelope_sender: Option<Sender>,
    pub first_recipient: Sender,
    pub headers: Option<String>,
    #[serde(rename = "_id")]
    pub id: (String, String),
    #[serde(with = "serde_base64")]
    pub list_unsubscribe: Vec<u8>,
    pub mail_details: (),
    pub mail_details_draft: (),
    #[serde(with = "serde_base64")]
    pub method: Vec<u8>,
    pub moved_time: String,
    #[serde(with = "serde_option_base64", rename = "_ownerEncSessionKey")] // with ?updateOwnerEncSessionKey=true
    pub owner_enc_session_key: Option<Vec<u8>>,
    #[serde(rename = "_ownerGroup")]
    pub owner_group: String,
    #[serde(rename = "_permissions")]
    pub permissions: String,
    pub phishing_status: String,
    pub received_date: String,
    pub recipient_count: String,
    pub reply_tos: Vec<Sender>,
    pub reply_type: String,
    pub restrictions: (),
    pub sent_date: String,
    pub sender: Sender,
    pub state: String,
    #[serde(with = "serde_base64")]
    pub subject: Vec<u8>,
    pub to_recipients: Vec<Sender>,
    pub unread: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Sender {
    pub address: String,
    pub contact: (),
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(with = "serde_base64")]
    pub name: Vec<u8>,
}

pub fn fetch_from_inbox(access_token: &str, mails: &str) -> Result<Vec<Mail>> {
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

pub fn fetch_from_id(
    access_token: &str,
    instance_list_id: &str,
    instance_id: &str,
) -> Result<Mail> {
    let url = url::Url::parse(super::BASE_URL)?
        .join(format!("/rest/tutanota/mail/{}/{}", instance_list_id, instance_id).as_str())?;

    let response = super::request::auth_get(url, access_token)
        .send()?
        .json::<Mail>()?;

    Ok(response)
}

pub fn update(access_token: &str, mail: &Mail) -> Result<()> {
    let url = url::Url::parse(super::BASE_URL)?
        .join(format!("/rest/tutanota/mail/{}/{}", mail.id.0, mail.id.1).as_str())?;

    let payload = serde_json::to_string(&mail)?;
    let _ = super::request::auth_put(url, access_token)
        .body(payload)
        .send()?
        .error_for_status()?;

    Ok(())
}
