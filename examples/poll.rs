use anyhow::Result;
use tracing::info;
use tuta_poll::client::Client;
use tuta_poll::*;
use tuta_poll::types::ReadStatus;

use futures_util::pin_mut;
use futures_util::StreamExt;


#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let mut email_address = String::new();
    println!("Please enter email: ");
    std::io::stdin()
        .read_line(&mut email_address)
        .expect("Expected email");
    email_address = email_address.trim().to_string();
    let password = rpassword::prompt_password("Password: ").expect("Expected password");

    let config = config::Account {
        email_address,
        password,
        watch_spam: false,
        show_name: true,
        show_subject: true,
        show_body: true,
    };

    let client = Client::new(&config).await?;

    let mails = client.get_mails();
    pin_mut!(mails);
    while let Some(mail) = mails.next().await {
        let mut mail = mail?;
        if mail.read_status == ReadStatus::Read {
            continue;
        }
        let decrypted_mail = client.decrypt(&mail).await;
        info!("Got mail: {:?}", decrypted_mail);
        client.mark_read(&mut mail).await?;
    }
    Ok(())
}
