use crate::http_client::{HttpClient, Method};
use crate::serialize::*;
use crate::types::Session;
use anyhow::Result;
use base64::{engine::general_purpose as engines, Engine as _};
use serde::Serialize;
use sha2::Digest;
use tracing::{debug, trace};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Request<'a> {
    #[serde(rename = "_format", with = "serde_format")]
    format: (),
    access_key: (),
    auth_token: (),
    auth_verifier: String,
    client_identifier: &'a str,
    mail_address: &'a str,
    recover_code_verifier: (),
    user: (),
}

pub async fn fetch(
    client: &HttpClient,
    email_address: &str,
    user_passphrase_key: &[u8],
) -> Result<Session> {
    debug!("Fetching session");

    let mut hasher = sha2::Sha256::new();
    hasher.update(user_passphrase_key);
    let hash = hasher.finalize();
    let auth_verifier = engines::URL_SAFE_NO_PAD.encode(&hash);
    let payload = serde_json::to_string(&Request {
        format: (),
        access_key: (),
        auth_token: (),
        auth_verifier,
        client_identifier: crate::api::CLIENT,
        mail_address: email_address,
        recover_code_verifier: (),
        user: (),
    })?;

    let url = url::Url::parse(super::BASE_URL)?.join("/rest/sys/sessionservice")?;

    let session = client
        .send(Method::Post, url, Some(payload))
        .await?
        .json::<Session>()
        .await?;
    debug!("Fetched session");
    trace!("session: {:#?}", session);
    Ok(session)
}
