use anyhow::Result;
use futures_util::pin_mut;
use futures_util::StreamExt;
use tracing::{info, warn};
use tuta_poll::client::Client;
use tuta_poll::types::ReadStatus;
use tuta_poll::*;

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
    let connector = client.get_websocket_connector()?;

    loop {
        info!("Connecting to websocket");
        let mut socket = connector.connect()?;

        while let Ok(has_new) = socket.has_new().await {
            if !has_new {
                continue;
            }
            let mails = client.get_mails();
            pin_mut!(mails);
            while let Some(mail) = mails.next().await {
                let mut mail = mail?;
                if mail.read_status == ReadStatus::Read {
                    continue;
                }
                let decrypted_mail = client.decrypt(&mail).await;
                info!("Got mail: {:?}", decrypted_mail);
                client.set_read_status(&mut mail, ReadStatus::Read).await?;
            }
        }
        warn!("Error getting mails. Retrying in 10s");
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
    // socket.close(None);
    // Ok(())
}
