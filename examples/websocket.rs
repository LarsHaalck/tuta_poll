use anyhow::{Error, Result};
use config::Config;
use lz4_flex::block::decompress_into;
use tracing::debug;
use tungstenite::Message;
use tuta_poll::mailfolder::MailFolderType;
use tuta_poll::user::GroupType;
use tuta_poll::*;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let config = Config::read("config.toml".into())?;

    let salt = salt::fetch(&config.account.email_address)?;
    let user_passphrase_key = crypto::create_user_passphrase_key(&config.account.password, &salt);
    let session = session::fetch(&config.account.email_address, &user_passphrase_key)?;

    let access_token = session.access_token;

    let mut url = url::Url::parse(BASE_URL)?.join("event")?;
    url.set_scheme("wss").unwrap();
    url.query_pairs_mut()
        .append_pair("modelVersions", &MODEL_VERSION)
        .append_pair("clientVersion", &CLIENT_VERSION)
        .append_pair("userId", &session.user)
        .append_pair("accessToken", &access_token);

    println!("url: {}", url.as_str());
    loop {
        let (mut socket, response) = tungstenite::connect(&url)?;
        println!("Connected to the server");
        println!("Response HTTP code: {}", response.status());
        println!("Response contains the following headers:");
        for (ref header, _value) in response.headers() {
            println!("* {}", header);
        }
        // loop {
        //     match socket.read() {
        //         Ok(m) => {}
        //         Err(e) => {
        //             println!("Error");
        //             std::thread::sleep(std::time::Duration::from_secs(5));
        //         }
        //     }
        // }
        while let Ok(msg) = socket.read() {
            match msg {
                Message::Text(text) => {
                    if let Some((a, b)) = text.split_once(";") {
                        match a {
                            "entityUpdate" => println!("Handle {} request with body {}", a, b),
                            _ => println!("Received unhandled response: {}", a),
                        }
                    }
                },
                Message::Binary(_) => println!("Got Binary"),
                Message::Ping(data) => {
                    println!("Got ping, answering with pong");
                    socket.write(Message::Pong(data))?;
                },
                Message::Pong(_) => println!("Got pong"),
                Message::Close(close_frame) => println!("Got close {:?}", close_frame),
                Message::Frame(_) => println!("Got frame"),
            }
        }
        println!("Error");
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
    // socket.close(None);
    // Ok(())
}
