use anyhow::{Error, Result};
use config::Config;
use lz4_flex::block::decompress_into;
use tuta_poll::mailfolder::MailFolderType;
use tuta_poll::user::GroupType;
use tuta_poll::*;
use tracing::debug;

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
    let root = mailboxgrouproot::fetch(&access_token, &mail_member.group)?;

    let mailbox = mailbox::fetch(&access_token, &root)?;
    let folders = mailfolder::fetch(&access_token, &mailbox)?;

    debug!("Selecting folders to watch");
    let inboxes: Vec<_> = if config.account.watch_spam {
        folders
            .iter()
            .filter(|folder| {
                folder.folder_type == MailFolderType::Inbox
                    || folder.folder_type == MailFolderType::Spam
            })
            .collect()
    } else {
        folders
            .iter()
            .filter(|folder| folder.folder_type == MailFolderType::Inbox)
            .collect()
    };

    debug!("Got {} folder(s)", inboxes.len());

    for inbox in &inboxes {
        debug!("Fetching mails from: {:?}", inbox.folder_type);
        let mails = mail::fetch(&access_token, &inbox.mails)?;
        for mut mail in mails {
            // skip read mails
            if mail.unread == "0" {
                debug!("Skipping already read email");
                continue;
            }

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

            // mark mail as read
            mail.unread = "0".to_string();
            mail::update(&access_token, &mail)?;
        }
    }

    Ok(())
}
