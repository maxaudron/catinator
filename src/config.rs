use serde::Deserialize;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Deserialize)]
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
            nick_password: Some(input.user.password),
            server: Some(input.server.hostname),
            port: Some(input.server.port),
            use_tls: Some(input.server.tls),
            channels: input.server.channels,
            ..irc::client::prelude::Config::default()
        }
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Deserialize)]
pub struct User {
    pub nickname: String,
    pub username: String,
    pub password: String,
    pub realname: String,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Deserialize)]
pub struct Server {
    pub hostname: String,
    pub port: u16,
    pub tls: bool,
    pub sasl: bool,
    pub channels: Vec<String>,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Deserialize)]
pub struct Settings {
    pub prefix: char,
}
