#![cfg_attr(all(test, feature = "bench"), feature(test))]
#[cfg(all(test, feature = "bench"))]
extern crate test;

use anyhow::{Context, Result};

use irc::client::prelude::*;

pub mod config;
pub mod hooks;
pub mod util;

pub use macros::catinator;

#[macro_export]
macro_rules! reply {
    ( $msg:expr, $text:expr ) => {
        bot.send_privmsg($msg.response_target().unwrap(), $text.as_str())?;
    };
}

pub struct Bot {
    pub config: config::Config,
    pub figment: figment::Figment,
    pub irc_client: irc::client::Client,
}

impl Bot {
    pub async fn new() -> Result<Bot> {
        let figment = config::Config::figment();
        let config: config::Config = figment.extract().context("failed to extract config")?;

        let irc_client = Client::from_config(config.clone().into()).await?;

        let bot = Bot { irc_client, config, figment };

        if bot.config.server.sasl && bot.config.user.password.is_some() {
            tracing::info!("initializing sasl");
            bot.sasl_init().await.unwrap()
        }

        Ok(bot)
    }

    pub fn figment(&self) -> &figment::Figment {
        &self.figment
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

    pub fn send_action(
        &self,
        target: &str,
        message: &str,
    ) -> std::result::Result<(), irc::error::Error> {
        self.irc_client.send_action(target, message)
    }
}
