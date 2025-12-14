//! The bots main configuration is housed in the [Config] struct.
//!
//! The configuration is by default loaded from `config.toml` in the current directory,
//! or from the path that is set by the `CATINATOR_CONFIG` variable. Environment
//! variables prefixed with `CATINATOR_` will also be loaded as configuration.
//!
//! The configuration uses [figment](https://docs.rs/figment) for it's configuration.
//!
//! # Profiles
//! The configuration is setup to use different profiles depending on how it is compiled.
//!
//! The main configuration file `config.toml` specifies the `default` profile which
//! sets default variables for most things. The `release` profile is also specified
//! in `config.toml` and selected when being built with the `--release` flag.
//! Additional configuration to set a different user while testing can be set in
//! the `debug` profile, which is selected if built without the `--release` flag.
//!
//! When compiled in release mode only the the `config.toml` or `CATINATOR_CONFIG`
//! and environment variables will be loaded.
//!
//! When compiled in debug mode an additional `config.debug.toml` is loaded
//! from the current directory. You can use this to set different channels and
//! nickname for testing.
//!
//! While developing this library / bot the `config.debug.toml` is also ignored from git.
//!
//! # Example
//! ```toml
//! [default]
//! [default.user]
//! # The username used for sasl and nickserv authentication
//! username = "catinator"
//! realname = "moaw"
//!
//! [default.server]
//! hostname = "<host>"
//! port = 6697
//! tls = true
//!
//! # Enabled sasl, also requires user.password to be set
//! sasl = true
//!
//! [default.settings]
//! # The prefix to use for commands
//! # Example: ":about"
//! prefix = ':'
//!
//! [release]
//! [release.user]
//! # The backslash has to be escaped here
//! nickname = "\\__{^-_-^}"
//! [release.server]
//! channels = ["<channel 1>", "<channel 2>"]
//! ```
//!
//! # Configuration for hooks
//!
//! If you write hooks that require some configuration you can use the
//! [`Bot::figment`](crate::Bot::figment) function to retrieve the figment and extract your own
//! configuration.
//!
//! ## Example:
//! ```
//! use serde::{Serialize, Deserialize};
//! use anyhow::{Context, Result};
//! use irc::client::prelude::*;
//! use figment::providers::Env;
//!
//! use catinator::Bot;
//!
//! // Define a struct to initialize that contains your configuration
//! // you need to derive serde's Serialize and Deserialize on it
//! #[derive(Clone, Debug, Deserialize, Serialize)]
//! pub struct WolframAlpha {
//!     wa_api_key: String,
//! }
//!
//! impl WolframAlpha {
//!     // Impl a `new()` function that gets passed a reference to the
//!     // bot, or just a reference to the figment.
//!     pub fn new(bot: &Bot) -> Result<WolframAlpha> {
//!         // Get the figment, we have to clone it here to merge our own env var config in.
//!         bot.figment
//!             .clone()
//!             .merge(Env::prefixed("CATINATOR_"))
//!             // Extract the config into the return type of this function
//!             .extract()
//!             .context("failed to extract wolfram alpha config")
//!     }
//!
//!     pub async fn wa(&self, bot: &Bot, msg: Message) -> Result<()> {
//!         // Impl your command, hook or matcher here to allow it to access it's configuration
//!
//!         Ok(())
//!     }
//! }
//! ```

use serde::{Deserialize, Serialize};

use figment::{
    providers::{Format, Toml},
    value::{Dict, Map},
    Error, Figment, Metadata, Profile, Provider,
};
use tracing::debug;

/// Configuration for the Bot
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct Config {
    /// Settings related to the [User]
    pub user: User,
    /// Settings related to the [Server]
    pub server: Server,
    /// General bot related [Settings]
    pub settings: Settings,
}

impl From<Config> for irc::client::prelude::Config {
    fn from(input: Config) -> Self {
        Self {
            nickname: Some(input.user.nickname),
            username: Some(input.user.username),
            realname: Some(input.user.realname),
            nick_password: input.user.password,
            password: input.server.password,
            server: Some(input.server.hostname),
            port: Some(input.server.port),
            use_tls: Some(input.server.tls),
            channels: input.server.channels,
            ..irc::client::prelude::Config::default()
        }
    }
}

/// User related configuration
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct User {
    /// The displayed nickname of the bot
    pub nickname: String,
    /// The username used for authentication
    pub username: String,
    /// The password used for authentication with nickserv
    /// Defaults to None
    #[serde(default)]
    pub password: Option<String>,
    /// The bots realname
    pub realname: String,
}

/// Connection info for the irc server
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct Server {
    /// Hostname to connect to
    pub hostname: String,
    /// Port to connect to (default: 6697 TLS)
    #[serde(default = "default_port")]
    pub port: u16,
    /// Wether or not to use TLS (default: true)
    #[serde(default = "default_tls")]
    pub tls: bool,
    /// Enable or disable sasl authentication (default: false)
    #[serde(default)]
    pub sasl: bool,
    /// The password for the server
    /// Defaults to None
    #[serde(default)]
    pub password: Option<String>,
    /// Channels to join (default: [])
    #[serde(default)]
    pub channels: Vec<String>,
}

const fn default_port() -> u16 {
    6697
}

const fn default_tls() -> bool {
    true
}

/// General settings for the bot
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct Settings {
    /// The prefix used for commands like `:about` (default: ':')
    #[serde(default = "default_prefix")]
    pub prefix: char,
    // pub wa_api_key: String,
}

const fn default_prefix() -> char {
    ':'
}

impl Config {
    /// Allow the configuration to be extracted from any [`figment::Provider`].
    pub fn from<T: Provider>(provider: T) -> Result<Config, Error> {
        Figment::from(provider).extract()
    }

    /// Provide a default provider, a `Figment`.
    pub fn figment() -> Figment {
        use figment::providers::Env;

        let mut figment = Figment::new();

        #[cfg(debug_assertions)]
        const PROFILE: &str = "debug";
        #[cfg(not(debug_assertions))]
        const PROFILE: &str = "release";

        let config_file = if let Ok(path) = std::env::var("CATINATOR_CONFIG") {
            path
        } else {
            "config.toml".to_string()
        };

        debug!("using config file: {}", config_file);

        figment = figment
            .merge(Toml::file(config_file).nested())
            .merge(Env::prefixed("CATINATOR_").split('_'));

        #[cfg(debug_assertions)]
        {
            debug!("sourcing debug config");
            figment = figment.merge(Toml::file("config.debug.toml").nested());
        }

        figment.select(PROFILE)
    }
}

// Make `Config` a provider itself for composability.
impl Provider for Config {
    fn metadata(&self) -> Metadata {
        Metadata::named("Library Config")
    }

    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        figment::providers::Serialized::defaults(self).data()
    }

    fn profile(&self) -> Option<Profile> {
        Some(Profile::Default)
    }
}
