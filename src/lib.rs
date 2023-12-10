const CLIENT: &'static str = "Firefox Browser";
const CLIENT_VERSION: &'static str = "3.118.30";
const MODEL_VERSION: &'static str = "90.64";

pub mod api;
pub mod client;
pub mod config;
pub mod types;
pub mod crypto;

mod serialize;
mod request;
