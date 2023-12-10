use anyhow::Result;
use tracing::{debug, info};
use tuta_poll::*;
use tuta_poll::client::Client;

fn main() -> Result<()> {
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

    let client = Client::new(&config)?;

    let mails = client.get_mails()?;
    let unread_mails : Vec<_> = mails.iter().filter(|m| m.unread == "1").collect();
    info!("Got {} mails, {} unread", mails.len(), unread_mails.len());
    for mail in unread_mails {
        if mail.unread == "0" {
            continue;
        }
        let decrypted_mail = client.decrypt(&mail);
        debug!("Got mail: {:?}", decrypted_mail);

        // client.mark_read(&mut mail)?;
    }
    Ok(())
}
