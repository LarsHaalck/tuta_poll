use anyhow::{Error, Result, bail};
use std::net::TcpStream;
use tracing::debug;
use tungstenite::Message;
use tungstenite::{protocol::WebSocket as TWebSocket, stream::MaybeTlsStream};

pub struct WebSocketConnector {
    url: url::Url,
}
pub struct WebSocket {
    socket: TWebSocket<MaybeTlsStream<TcpStream>>,
}

#[derive(Debug)]
pub struct EntityUpdate {}

impl WebSocketConnector {
    pub fn from_url(access_token: &str, user_id: &str) -> Result<WebSocketConnector> {
        let mut url = url::Url::parse(super::BASE_URL)?.join("event")?;
        url.set_scheme("wss")
            .map_err(|e| Error::msg(format!("Could not set scheme to wss with error {:?}", e)))?;
        url.query_pairs_mut()
            .append_pair("modelVersions", &super::MODEL_VERSION)
            .append_pair("clientVersion", &super::CLIENT_VERSION)
            .append_pair("userId", user_id)
            .append_pair("accessToken", access_token);
        Ok(WebSocketConnector { url })
    }

    pub fn connect(&self) -> Result<WebSocket> {
        let (socket, response) = tungstenite::connect(&self.url)?;
        debug!("Connected to the server");
        debug!("Response HTTP code: {}", response.status());
        debug!("Response contains the following headers:");
        for (ref header, _value) in response.headers() {
            debug!("* {}", header);
        }

        Ok(WebSocket { socket })
    }
}

impl WebSocket {
    pub fn read(&mut self) -> Result<EntityUpdate> {
        loop {
            if let Ok(msg) = self.socket.read() {
                match msg {
                    Message::Text(text) => {
                        if let Some((a, b)) = text.split_once(";") {
                            match a {
                                "entityUpdate" => {
                                    debug!("Handle {} request with body {}", a, b);
                                    return Ok(EntityUpdate{});
                                },
                                _ => debug!("Received ignored response: {}", a),
                            }
                        }
                    }
                    Message::Binary(_) => debug!("Got pinary reponse"),
                    Message::Ping(data) => {
                        debug!("Got ping, answering with pong");
                        self.socket.write(Message::Pong(data))?;
                    }
                    Message::Pong(_) => debug!("Got pong"),
                    Message::Close(close_frame) => {
                        debug!("Got close {:?}", close_frame);
                        bail!("Connection closed, needs to be reconnected");
                    }
                    Message::Frame(_) => debug!("Got frame"),
                }
            }
        }
    }
}