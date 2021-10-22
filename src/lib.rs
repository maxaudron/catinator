//! catinator is a general purpose irc bot making crate.
//!
//! Most of the heavy lifting is done by the [irc](https://docs.rs/irc) crate,
//! but catinator adds useful high level utilities to make the bot making easier.
//!
//! Example:
//! ```no_run
//! #[tokio::main]
//! async fn main() {
//!     tracing_subscriber::fmt::init();
//!
//!     // Initialize the bot, loading the config and establishing the connection
//!     let mut bot = catinator::Bot::new().await.unwrap();
//!
//!     // Setup any modules that require it.
//!     let wolfram_alpha = catinator::hooks::wolfram_alpha::WolframAlpha::new(&bot)
//!         .expect("failed to initialize WolframAlpha command");
//!
//!     // Call the catinator macro to setup the hooks, matchers and commands
//!     catinator::catinator![
//!         // For example add the sasl hook to handle sasl authentication
//!         hook("sasl", "Handle Authentication.", AUTHENTICATE, catinator::hooks::sasl),
//!
//!         // Add a matcher that executes on a specific regex
//!         matcher("shifty_eyes", ">.>", r"^\S{3}$", catinator::hooks::shifty_eyes),
//!
//!         // Add an async command that calls a method on the previously instantiated struct.
//!         async command("wa", "Returns Wolfram Alpha results for a query", wolfram_alpha.wa),
//!     ];
//! }
//! ```

#![cfg_attr(all(test, feature = "bench"), feature(test))]
#[cfg(all(test, feature = "bench"))]
extern crate test;

use anyhow::{Context, Result};

use irc::client::prelude::*;

pub mod config;
pub mod hooks;
pub mod util;

// Rexport of the catinator proc macros
pub use macros::catinator;

/// The struct handling bot actions and configuration
pub struct Bot {
    /// The base config of the bot
    pub config: config::Config,
    /// The figment the config is extracted from
    pub figment: figment::Figment,
    /// The irc client object, used to send messages etc
    /// It is recommended to use the methods directly on the Bot struct instead.
    pub irc_client: irc::client::Client,
}

impl Bot {
    /// Initializes the bot.
    /// Loads configuration from `CATINATOR_` environment variables and the `config.toml` file
    /// Starts the connection to the irc server.
    pub async fn new() -> Result<Bot> {
        let figment = config::Config::figment();
        let config: config::Config = figment.extract().context("failed to extract config")?;

        let irc_client = Client::from_config(config.clone().into()).await?;

        let bot = Bot {
            irc_client,
            config,
            figment,
        };

        if bot.config.server.sasl && bot.config.user.password.is_some() {
            tracing::info!("initializing sasl");
            bot.sasl_init().await.unwrap()
        }

        Ok(bot)
    }

    /// Get the bots figment to use when building your own configuration.
    /// See [config]
    pub fn figment(&self) -> &figment::Figment {
        &self.figment
    }

    /// Initialize a sasl connection, you usually don't need
    /// to run this yourself as it is done during [Bot::new].
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

    /// Send a privmsg to the target `#channel` or `user`
    pub fn send_privmsg(
        &self,
        target: &str,
        message: &str,
    ) -> std::result::Result<(), irc::error::Error> {
        self.irc_client.send_privmsg(target, message)
    }

    /// Send a notice to the target `#channel` or `user`
    pub fn send_notice(
        &self,
        target: &str,
        message: &str,
    ) -> std::result::Result<(), irc::error::Error> {
        self.irc_client.send_notice(target, message)
    }

    /// Send an action (`/me`) to the target `#channel` or `user`
    pub fn send_action(
        &self,
        target: &str,
        message: &str,
    ) -> std::result::Result<(), irc::error::Error> {
        self.irc_client.send_action(target, message)
    }
}
