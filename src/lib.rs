//! A small library to poll mails from tutanota and possibly decrypt sender name, subject and body and mark them as read.
//!
//! ## Overview
//! The library provides a high level client with the option to connect to a websocket to listen for new messages as well as a wrapper for the undocumented tuta api.
//! The library can also decrypt sender name, subject and message body.
//!
//! ## Usage
//!
//! ### Config and Client
//!
//! ```ignore
//! let config = tuta_poll::config::Account {
//!     "...@tuta.com",
//!     "some_password",
//!     watch_spam: true,
//!     show_name: true,
//!     show_subject: true,
//!     show_body: true,
//! };
//!
//! let client = tuta_poll::client::Client::new(&config).await?;
//! ```
//!
//! ### Get messages
//! ```ignore
//! use futures_util::pin_mut;
//! use futures_util::StreamExt;
//!
//! let mails = client.get_mails();
//! pin_mut!(mails);
//! while let Some(mail) = mails.next().await {
//!     let mut mail = mail?;
//!     let decrypted_mail = client.decrypt(&mail).await;
//! }
//! ```
//! ### Connect to websocket
//!
//! ```ignore
//! let connector = client.get_websocket_connector()?;
//!
//! loop {
//!     let mut socket = connector.connect()?;
//!     while let Ok(has_new) = socket.has_new().await {
//!         if !has_new {
//!             continue;
//!         }
//!         let mails = client.get_mails();
//!     }
//! }
//! ```

pub mod api;
pub mod client;
pub mod config;
pub mod crypto;
pub mod types;

mod http_client;
mod serialize;
mod websocket;
