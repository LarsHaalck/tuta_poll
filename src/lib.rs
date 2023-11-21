const BASE_URL: &'static str = "https://app.tuta.com";
const CLIENT: &'static str = "Firefox Browser";

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
