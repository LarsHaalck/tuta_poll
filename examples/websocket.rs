use anyhow::{Error, Result};
use config::Config;
use tuta_poll::*;

use tuta_poll::websocket::OperationType;
use tuta_poll::user::GroupType;
use lz4_flex::block::decompress_into;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let config = Config::read("config.toml".into())?;

    let salt = salt::fetch(&config.account.email_address)?;
    let user_passphrase_key = crypto::create_user_passphrase_key(&config.account.password, &salt);
    let session = session::fetch(&config.account.email_address, &user_passphrase_key)?;

    let access_token = session.access_token;
    let user = user::fetch(&access_token, &session.user)?;

    let mail_member = user
        .memberships
        .iter()
        .find(|membership| membership.group_type == GroupType::Mail)
        .ok_or(Error::msg("Could not find group with type mail"))?;

    let user_group_key = crypto::decrypt_key(&user_passphrase_key, &user.user_group.sym_enc_g_key)?;
    let mail_group_key = crypto::decrypt_key(&user_group_key, &mail_member.sym_enc_g_key)?;

    let connector = websocket::WebSocketConnector::from_url(&access_token, &session.user)?;

    loop {
        let mut socket = connector.connect()?;

        while let Ok(update) = socket.read() {
            println!("Update: {:?}", update);

            let events: Vec<_> = update
                .event_batch
                .iter()
                .filter(|b| b.operation == OperationType::Create)
                .collect();
            let mail_ids: Vec<_> = events
                .iter()
                .filter(|b| b.event_type == "Mail")
                .collect();

            println!("Mail: {:?}", mail_ids);
            for mail_id in mail_ids {
                let mail =
                    mail::fetch_from_id(&access_token, &mail_id.instance_list_id, &mail_id.instance_id)?;
                let session_key =
                    crypto::decrypt_key(&mail_group_key, &mail.owner_enc_session_key).unwrap();
                let session_sub_keys = crypto::SubKeys::new(session_key);

                let subject = if config.display.subject {
                    let tmp = crypto::decrypt_with_mac(&session_sub_keys, &mail.subject)?;
                    std::str::from_utf8(&tmp)
                        .expect("Subject could not converted to UTF-8")
                        .to_string()
                } else {
                    "∅".to_string()
                };

                let name = if config.display.name {
                    let tmp = crypto::decrypt_with_mac(&session_sub_keys, &mail.sender.name)?;
                    std::str::from_utf8(&tmp)
                        .expect("Name could not converted to UTF-8")
                        .to_string()
                } else {
                    "∅".to_string()
                };

                let address = if config.display.name {
                    mail.sender.address.to_string()
                } else {
                    "∅".to_string()
                };

                println!(
                    "new mail, subject: {:?}, from: {:?} <{:?}>, [{} attachments]",
                    subject, name, address, mail.attachments.len()
                );

                if config.display.body {
                    let mailbody = mailbody::fetch(&access_token, &mail.body)?;
                    let compressed_text = crypto::decrypt_with_mac(&session_sub_keys, &mailbody)?;
                    let mut buf: Vec<u8> = vec![0; mailbody.len() * 6];
                    let size = decompress_into(&compressed_text, &mut buf)?;
                    buf.resize(size, 0);
                    println!("mail body: {}", std::str::from_utf8(&buf).unwrap());
                }

            }
        }
        println!("Error");
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
    // socket.close(None);
    // Ok(())
}
