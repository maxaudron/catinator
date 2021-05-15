#![feature(test)]
extern crate test;

use anyhow::Result;

use irc::client::prelude::*;

use tracing::info;

pub mod config;
pub mod hooks;

pub use macros::catinator;

#[macro_export]
macro_rules! reply {
    ( $msg:expr, $text:expr ) => {
        bot.send_privmsg($msg.response_target().unwrap(), $text.as_str())?;
    };
}


pub struct Bot {
    pub config: config::Config,
    pub irc_client: irc::client::Client,
}

impl Bot {
    pub async fn new(config_path: &str) -> Result<Bot> {
        use std::fs;

        let config_str = fs::read_to_string(config_path)?;
        let mut config: config::Config = toml::from_str(&config_str)?;

        match std::env::var("CATINATOR_PASSWORD") {
            Ok(var) => {
                info!("using password from env var");
                config.user.password = var
            }
            Err(_) => ()
        }

        let irc_client = Client::from_config(config.clone().into()).await?;

        Ok(Bot { irc_client, config })
    }

    pub async fn sasl_init(&self) -> Result<()> {
        self.irc_client
            .send_cap_req(&vec![irc::client::prelude::Capability::Sasl])?;
        self.irc_client
            .send(Command::NICK(self.config.user.nickname.clone()))?;
        self.irc_client.send(Command::USER(
            self.config.user.nickname.clone(),
            "0".to_owned(),
            self.config.user.realname.clone(),
        ))?;
        self.irc_client.send_sasl_plain()?;

        Ok(())
    }

    pub fn send_privmsg(
        &self,
        target: &str,
        message: &str,
    ) -> std::result::Result<(), irc::error::Error> {
        self.irc_client.send_privmsg(target, message)
    }

    pub fn send_notice(
        &self,
        target: &str,
        message: &str,
    ) -> std::result::Result<(), irc::error::Error> {
        self.irc_client.send_notice(target, message)
    }
}
