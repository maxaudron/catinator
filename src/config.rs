use serde::{Deserialize, Serialize};

use figment::{
    providers::{Format, Toml},
    value::{Dict, Map},
    Error, Figment, Metadata, Profile, Provider,
};
use tracing::debug;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct Config {
    pub user: User,
    pub server: Server,
    pub settings: Settings,
}

impl From<Config> for irc::client::prelude::Config {
    fn from(input: Config) -> Self {
        Self {
            nickname: Some(input.user.nickname),
            username: Some(input.user.username),
            realname: Some(input.user.realname),
            nick_password: input.user.password,
            server: Some(input.server.hostname),
            port: Some(input.server.port),
            use_tls: Some(input.server.tls),
            channels: input.server.channels,
            ..irc::client::prelude::Config::default()
        }
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct User {
    pub nickname: String,
    pub username: String,
    #[serde(default)]
    pub password: Option<String>,
    pub realname: String,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct Server {
    pub hostname: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_tls")]
    pub tls: bool,
    #[serde(default)]
    pub sasl: bool,
    #[serde(default)]
    pub channels: Vec<String>,
}

const fn default_port() -> u16 {
    6697
}

const fn default_tls() -> bool {
    true
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct Settings {
    #[serde(default = "default_prefix")]
    pub prefix: char,
    // pub wa_api_key: String,
}

const fn default_prefix() -> char {
    ':'
}

impl Config {
    // Allow the configuration to be extracted from any `Provider`.
    pub fn from<T: Provider>(provider: T) -> Result<Config, Error> {
        Figment::from(provider).extract()
    }

    // Provide a default provider, a `Figment`.
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
