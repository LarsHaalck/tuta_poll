use anyhow::{Context, Error, Result};
use serde::Deserialize;
use tracing::debug;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub account: Account,
    pub display: Display,
}

#[derive(Deserialize, Debug)]
pub struct Account {
    pub email_address: String,
    pub password: String,
}

#[derive(Deserialize, Debug)]
pub struct Display {
    #[serde(default)] // default false
    pub email: bool,
    #[serde(default)]
    pub name: bool,
    #[serde(default)]
    pub subject: bool,
    #[serde(default)]
    pub body: bool,
}

impl Config {
    pub fn read(config_file: std::path::PathBuf) -> Result<Config> {
        let config: Option<Config>;
        // try to read file first
        if config_file.is_file() {
            debug!("Trying to read config from file: {}", config_file.display());
            let config_str =
                std::fs::read_to_string(config_file).context("Could not read config file")?;
            config =
                Some(toml::from_str(config_str.as_str()).context("Could not parse config file")?);
        } else {
            debug!("Trying to read config from env");
            let account = envy::prefixed("TP_ACCOUNT_").from_env::<Account>()?;
            let display = envy::prefixed("TP_DISPLAY_").from_env::<Display>()?;
            config = Some(Config { account, display });
        }

        config.ok_or(Error::msg(
            "Could not read config file from either file nor environment",
        ))
    }
}
