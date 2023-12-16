use crate::http_client::{HttpClient, Method};
use crate::types::{Id, Mail};
use anyhow::Result;
use tracing::{debug, trace};

pub async fn fetch_from_inbox(
    client: &HttpClient,
    mails: &str,
    start: Option<Id>,
) -> Result<Vec<Mail>> {
    debug!("Fetching mails");
    let mut url = url::Url::parse(super::BASE_URL)?
        .join(format!("/rest/tutanota/mail/{}", mails).as_str())?;
    url.query_pairs_mut()
        .append_pair("count", "100")
        .append_pair("reverse", "true");
    if let Some(s) = start {
        url.query_pairs_mut().append_pair("start", &s);
    } else {
        url.query_pairs_mut().append_pair("start", "zzzzzzzzzzzz");
    }

    let mails = client
        .send(Method::AuthGet, url, None)
        .await?
        .json::<Vec<Mail>>()
        .await?;

    debug!("Fetched {} mails", mails.len());
    trace!("mails: {:#?}", mails);
    Ok(mails)
}

pub async fn fetch_from_id(
    client: &HttpClient,
    instance_list_id: &str,
    instance_id: &str,
) -> Result<Mail> {
    debug!("Fetching single mail");
    let url = url::Url::parse(super::BASE_URL)?
        .join(format!("/rest/tutanota/mail/{}/{}", instance_list_id, instance_id).as_str())?;

    let mail = client
        .send(Method::AuthGet, url, None)
        .await?
        .json::<Mail>()
        .await?;
    debug!("Fetched single mail");
    trace!("mail: {:#?}", mail);
    Ok(mail)
}

pub async fn update(client: &HttpClient, mail: &Mail, update_key: bool) -> Result<()> {
    let mut url = url::Url::parse(super::BASE_URL)?
        .join(format!("/rest/tutanota/mail/{}/{}", mail.id.0, mail.id.1).as_str())?;

    if update_key {
        url.query_pairs_mut()
            .append_pair("updateOwnerEncSessionKey", "true");
    }

    let payload = serde_json::to_string(&mail)?;
    client.send(Method::AuthPut, url, Some(payload)).await?;

    Ok(())
}
