use anyhow::{Error, Result};
use config::Config;
use tuta_poll::*;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let config = Config::read("config.toml".into())?;

    let salt = salt::fetch(&config.account.email_address)?;
    let user_passphrase_key = crypto::create_user_passphrase_key(&config.account.password, &salt);
    let session = session::fetch(&config.account.email_address, &user_passphrase_key)?;

    let access_token = session.access_token;
    let connector = websocket::WebSocketConnector::from_url(&access_token, &session.user)?;

    loop {
        let mut socket = connector.connect()?;

        while let Ok(update) = socket.read() {
            println!("Update: {:?}", update);
        }
        println!("Error");
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
    // socket.close(None);
    // Ok(())
}
