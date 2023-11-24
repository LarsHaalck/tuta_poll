use anyhow::{Error, Result};
use inquire::Confirm;
use lz4_flex::block::decompress_into;
use tracing::debug;
use tuta_poll::mailfolder::MailFolderType;
use tuta_poll::user::GroupType;
use tuta_poll::*;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let mut email_address = String::new();
    println!("Please enter email: ");
    std::io::stdin()
        .read_line(&mut email_address)
        .expect("Expected email");
    email_address = email_address.trim().to_string();
    let password = rpassword::prompt_password("Password: ").expect("Expected password");

    let salt = salt::fetch(&email_address)?;
    let user_passphrase_key = crypto::create_user_passphrase_key(&password, &salt);
    let session = session::fetch(&email_address, &user_passphrase_key)?;

    let access_token = session.access_token;
    let user = user::fetch(&access_token, &session.user)?;

    let mail_member = user
        .memberships
        .iter()
        .find(|membership| membership.group_type == GroupType::Mail)
        .ok_or(Error::msg("Could not find group with type mail"))?;

    let user_group_key = crypto::decrypt_key(&user_passphrase_key, &user.user_group.sym_enc_g_key)?;
    let mail_group_key = crypto::decrypt_key(&user_group_key, &mail_member.sym_enc_g_key)?;
    let root = mailboxgrouproot::fetch(&access_token, &mail_member.group)?;

    let mailbox = mailbox::fetch(&access_token, &root)?;
    let folders = mailfolder::fetch(&access_token, &mailbox)?;

    let inboxes: Vec<_> = folders
        .iter()
        .filter(|folder| folder.folder_type == MailFolderType::Inbox)
        .collect();

    for inbox in &inboxes {
        debug!("Fetching mails from: {:?}", inbox.folder_type);
        let mails = mail::fetch_from_inbox(&access_token, &inbox.mails)?;
        for mut mail in mails {
            // skip read mails
            if mail.unread == "0" {
                debug!("Skipping already read email");
                continue;
            }

            // owner_enc_session_key should also be Some
            let session_key = crypto::decrypt_key(
                &mail_group_key,
                &mail.owner_enc_session_key.as_ref().unwrap(),
            )
            .expect("Could not retrieve session key");
            let session_sub_keys = crypto::SubKeys::new(session_key);

            let subject =
                if let Ok(true) = Confirm::new("Show subject?").with_default(false).prompt() {
                    let tmp = crypto::decrypt_with_mac(&session_sub_keys, &mail.subject)?;
                    std::str::from_utf8(&tmp)
                        .expect("Subject could not converted to UTF-8")
                        .to_string()
                } else {
                    "∅".to_string()
                };

            let name = if let Ok(true) = Confirm::new("Show name?").with_default(false).prompt() {
                let tmp = crypto::decrypt_with_mac(&session_sub_keys, &mail.sender.name)?;
                std::str::from_utf8(&tmp)
                    .expect("Name could not converted to UTF-8")
                    .to_string()
            } else {
                "∅".to_string()
            };

            println!(
                "new mail, subject: {:?}, from: {:?} <{:?}>, [{} attachments]",
                subject,
                name,
                mail.sender.address.to_string(),
                mail.attachments.len()
            );

            if let Ok(true) = Confirm::new("Show body?").with_default(false).prompt() {
                let mailbody = mailbody::fetch(&access_token, &mail.body)?;
                let compressed_text = crypto::decrypt_with_mac(&session_sub_keys, &mailbody)?;
                let mut buf: Vec<u8> = vec![0; mailbody.len() * 6];
                let size = decompress_into(&compressed_text, &mut buf)?;
                buf.resize(size, 0);
                println!("mail body: {}", std::str::from_utf8(&buf).unwrap());
            }

            // mark mail as read
            mail.unread = "0".to_string();
            mail::update(&access_token, &mail)?;
        }
    }

    Ok(())
}
