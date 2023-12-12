use anyhow::Result;
use tracing::{debug, trace};
use crate::types::Mail;

pub fn fetch_from_inbox(access_token: &str, mails: &str) -> Result<Vec<Mail>> {
    debug!("Fetching mails");
    let mut url = url::Url::parse(super::BASE_URL)?
        .join(format!("/rest/tutanota/mail/{}", mails).as_str())?;
    url.query_pairs_mut()
        .append_pair("start", "zzzzzzzzzzzz")
        .append_pair("count", "1000")
        .append_pair("reverse", "true");

    let mails = crate::request::auth_get(url.clone(), access_token)
        .send()?
        .error_for_status()?
        .json::<Vec<Mail>>()?;

    debug!("Fetched {} mails", mails.len());
    trace!("mails: {:#?}", mails);
    Ok(mails)
}

pub fn fetch_from_id(
    access_token: &str,
    instance_list_id: &str,
    instance_id: &str,
) -> Result<Mail> {
    debug!("Fetching single mail");
    let url = url::Url::parse(super::BASE_URL)?
        .join(format!("/rest/tutanota/mail/{}/{}", instance_list_id, instance_id).as_str())?;

    let mail = crate::request::auth_get(url, access_token)
        .send()?
        .error_for_status()?
        .json::<Mail>()?;
    debug!("Fetched single mail");
    trace!("mail: {:#?}", mail);
    Ok(mail)
}

pub fn update(access_token: &str, mail: &Mail, update_key: bool) -> Result<()> {
    let mut url = url::Url::parse(super::BASE_URL)?
        .join(format!("/rest/tutanota/mail/{}/{}", mail.id.0, mail.id.1).as_str())?;

    if update_key {
        url.query_pairs_mut()
            .append_pair("updateOwnerEncSessionKey", "true");
    }

    let payload = serde_json::to_string(&mail)?;
    let _ = crate::request::auth_put(url, access_token)
        .body(payload)
        .send()?
        .error_for_status()?;

    Ok(())
}
