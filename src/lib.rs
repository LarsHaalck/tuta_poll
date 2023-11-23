pub const BASE_URL: &'static str = "https://app.tuta.com";
pub const CLIENT: &'static str = "Firefox Browser";

pub const CLIENT_VERSION: &'static str = "3.118.30";
pub const MODEL_VERSION: &'static str = "90.64";

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

mod serialize;
mod request;
