use crate::http_client::HttpClient;
use crate::types::{EntityUpdate, OperationType};
use anyhow::{anyhow, bail, Context, Result};
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

impl WebSocketConnector {
    pub fn from_url(client: &HttpClient, user_id: &str) -> Result<WebSocketConnector> {
        let mut url = url::Url::parse(crate::api::BASE_URL)?.join("event")?;
        url.set_scheme("wss")
            .map_err(|e| anyhow!("Could not set scheme to wss with error {:?}", e))?;
        url.query_pairs_mut()
            .append_pair("modelVersions", &crate::api::MODEL_VERSION)
            .append_pair("clientVersion", &crate::api::CLIENT_VERSION)
            .append_pair("userId", user_id)
            .append_pair(
                "accessToken",
                &client
                    .get_access_token()
                    .context("Client must be authenticated first")?,
            );
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
    pub async fn has_new(&mut self) -> Result<bool> {
        let update = self.read_all()?;
        let events: Vec<_> = update
            .event_batch
            .iter()
            .filter(|b| b.operation == OperationType::Create)
            .filter(|b| b.event_type == "Mail")
            .collect();

        if events.is_empty() {
            Ok(false)
        } else {
            Ok(true)
        }
    }
    fn read_all(&mut self) -> Result<EntityUpdate> {
        loop {
            if let Ok(msg) = self.socket.read() {
                match msg {
                    Message::Text(text) => {
                        if let Some((a, b)) = text.split_once(";") {
                            match a {
                                "entityUpdate" => {
                                    debug!("Handle {} request", a);
                                    return Ok(serde_json::from_str(b)?);
                                }
                                _ => debug!("Received ignored response: {}", a),
                            }
                        }
                    }
                    Message::Binary(_) => debug!("Got binary reponse"),
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

    pub fn close(&mut self) -> Result<()> {
        self.socket.close(None)?;
        Ok(())
    }
}
