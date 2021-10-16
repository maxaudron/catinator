#![cfg_attr(all(test, feature = "bench"), feature(test))]
#[cfg(all(test, feature = "bench"))]
extern crate test;

use anyhow::Result;

use irc::client::prelude::*;

use tracing::info;

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
    pub irc_client: irc::client::Client,
}

fn get_env_var(var_name: &str) -> Option<String> {
    match std::env::var(var_name) {
        Ok(var) => {
            info!("using {} from env", var_name);
            Some(var)
        }
        Err(_) => None,
    }
}

impl Bot {
    pub async fn new(config_path: &str) -> Result<Bot> {
        use std::fs;

        let config_str = fs::read_to_string(config_path)?;
        let mut config: config::Config = toml::from_str(&config_str)?;
        let bot = Bot { irc_client, config, figment };

        if let Some(v) = get_env_var("CATINATOR_PASSWORD") {
            config.user.password = v
        };
        if bot.config.server.sasl && bot.config.user.password.is_some() {
            tracing::info!("initializing sasl");
            bot.sasl_init().await.unwrap()
        }

        if let Some(v) = get_env_var("CATINATOR_WA_API_KEY") {
            config.settings.wa_api_key = v
        };

        let irc_client = Client::from_config(config.clone().into()).await?;
        Ok(bot)
    }

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

    pub fn send_action(
        &self,
        target: &str,
        message: &str,
    ) -> std::result::Result<(), irc::error::Error> {
        self.irc_client.send_action(target, message)
    }
}
