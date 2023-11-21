use anyhow::{Result, Error};
use tuta_poll::config::Config;
use lz4_flex::block::decompress_into;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let config = Config::read("config.toml".into())?;
    println!("Config: {:?}", config);

    let salt = tuta_poll::salt::fetch(&config.account.email_address)?;
    println!("salt: {:?}", salt);
    let user_passphrase_key =
        tuta_poll::crypto::create_user_passphrase_key(&config.account.password, &salt);
    println!("user_passphrase_key: {:?}", user_passphrase_key);
    let session = tuta_poll::session::fetch(&config.account.email_address, &user_passphrase_key)?;
    println!("session: {:?}", session);

    let access_token = session.access_token;
    let user = tuta_poll::user::fetch(&access_token, &session.user)?;
    println!("user: {:?}", user);

    let mail_member = user
        .memberships
        .iter()
        .find(|membership| membership.group_type == "5").ok_or(Error::msg("Could not find group with type mail"))?;

    let user_group_key = tuta_poll::crypto::decrypt_key(
        &user_passphrase_key,
        &user.user_group.sym_enc_g_key,
    )?;
    let mail_group_key = tuta_poll::crypto::decrypt_key(
        &user_group_key,
        &mail_member.sym_enc_g_key,
    )?;
    let root = tuta_poll::mailboxgrouproot::fetch(
        &access_token,
        &mail_member.group,
    )?;

    let mailbox = tuta_poll::mailbox::fetch(&access_token, &root)?;
    println!("mailbox: {:?}", mailbox);
    let folders = tuta_poll::mailfolder::fetch(&access_token, &mailbox)?;
    println!("folders: {:?}", folders);

    let inbox = folders
        .iter()
        .find(|folder| folder.folder_type == "1").ok_or(Error::msg("Could not find inbox"))?;
    let mails = tuta_poll::mail::fetch(&access_token, &inbox.mails)?;
    for mail in &mails {
        let session_key =
            tuta_poll::crypto::decrypt_key(&mail_group_key, &mail.owner_enc_session_key).unwrap();
        let session_sub_keys = tuta_poll::crypto::SubKeys::new(session_key);

        let title =
            tuta_poll::crypto::decrypt_with_mac(&session_sub_keys, &mail.subject)?;
        let display_name =
            tuta_poll::crypto::decrypt_with_mac(&session_sub_keys, &mail.sender.name)?;
        println!(
            "mail, subject: {:?}, from: {:?} <{:?}>",
            std::str::from_utf8(&title).unwrap(),
            std::str::from_utf8(&display_name).unwrap(),
            mail.sender.address,
        );

        println!("Num attachments: {}", mail.attachments.len());
        let mailbody = tuta_poll::mailbody::fetch(&access_token, &mail.body)?;
        let compressed_text = tuta_poll::crypto::decrypt_with_mac(&session_sub_keys, &mailbody)?;
        let mut buf : Vec<u8> = vec![0; mailbody.len() * 6];
        let size = decompress_into(&compressed_text, &mut buf)?;
        buf.resize(size, 0);
        println!("mail body: {}", std::str::from_utf8(&buf).unwrap());
    }

    Ok(())
}
