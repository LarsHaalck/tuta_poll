use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Account {
    pub email_address: String,
    pub password: String,
    #[serde(default)]
    pub watch_spam: bool,
    #[serde(default)]
    pub show_name: bool,
    #[serde(default)]
    pub show_subject: bool,
    #[serde(default)]
    pub show_body: bool,
}

