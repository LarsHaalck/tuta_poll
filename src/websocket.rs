use crate::api;
use crate::types::{EntityUpdate, Mail, OperationType};
use anyhow::{bail, Error, Result};
use std::net::TcpStream;
use tracing::debug;
use tungstenite::Message;
use tungstenite::{protocol::WebSocket as TWebSocket, stream::MaybeTlsStream};

pub struct WebSocketConnector<'a> {
    url: url::Url,
    access_token: &'a str,
    inboxes: &'a Vec<String>,
}

pub struct WebSocket<'a> {
    socket: TWebSocket<MaybeTlsStream<TcpStream>>,
    access_token: &'a str,
    inboxes: &'a Vec<String>,
}

impl WebSocketConnector<'_> {
    pub fn from_url<'a>(
        access_token: &'a str,
        user_id: &str,
        inboxes: &'a Vec<String>,
    ) -> Result<WebSocketConnector<'a>> {
        let mut url = url::Url::parse(crate::api::BASE_URL)?.join("event")?;
        url.set_scheme("wss")
            .map_err(|e| Error::msg(format!("Could not set scheme to wss with error {:?}", e)))?;
        url.query_pairs_mut()
            .append_pair("modelVersions", &crate::api::MODEL_VERSION)
            .append_pair("clientVersion", &crate::api::CLIENT_VERSION)
            .append_pair("userId", user_id)
            .append_pair("accessToken", access_token);
        Ok(WebSocketConnector {
            url,
            access_token: &access_token,
            inboxes: &inboxes,
        })
    }

    pub fn connect(&self) -> Result<WebSocket> {
        let (socket, response) = tungstenite::connect(&self.url)?;
        debug!("Connected to the server");
        debug!("Response HTTP code: {}", response.status());
        debug!("Response contains the following headers:");
        for (ref header, _value) in response.headers() {
            debug!("* {}", header);
        }

        Ok(WebSocket {
            socket,
            access_token: &self.access_token,
            inboxes: &self.inboxes,
        })
    }
}

impl WebSocket<'_> {
    pub async fn read_create(&mut self) -> Result<Vec<Mail>> {
        let update = self.read_all()?;
        let events: Vec<_> = update
            .event_batch
            .iter()
            .filter(|b| b.operation == OperationType::Create)
            .filter(|b| b.event_type == "Mail")
            .collect();

        if events.is_empty() {
            Ok(Vec::new())
        } else {
            let mut mails = Vec::new();
            for inbox in self.inboxes {
                mails.extend(api::mail::fetch_from_inbox(&self.access_token, &inbox).await?);
            }
            Ok(mails)
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

    pub fn close(&mut self) -> Result<()> {
        self.socket.close(None)?;
        Ok(())
    }
}
