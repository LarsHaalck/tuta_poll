use anyhow::Result;
use tracing::{info, warn};
use tuta_poll::client::Client;
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

        while let Ok(mut mails) = socket.read_create().await {
            info!("Got batch of {} mails", mails.len());
            for mail in &mut mails {
                let decrypted_mail = client.decrypt(&mail).await;
                info!("Got mail: {:?}", decrypted_mail);
                client.mark_read(mail).await?;
            }
        }
        warn!("Error getting mails. Retrying in 10s");
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
    // socket.close(None);
    // Ok(())
}
