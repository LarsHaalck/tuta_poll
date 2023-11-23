const BASE_URL: &'static str = "https://app.tuta.com";
const CLIENT: &'static str = "Firefox Browser";
const CLIENT_VERSION: &'static str = "3.118.30";
const MODEL_VERSION: &'static str = "90.64";

pub mod config;
pub mod salt;
pub mod crypto;
pub mod session;
pub mod user;
pub mod mailbox;
pub mod mailboxgrouproot;
pub mod mailfolder;
pub mod mail;
pub mod mailbody;
pub mod websocket;

mod serialize;
mod request;
