//! The bots hooks, commands and matchers. For explanation of different types see [crate::catinator]
//!
//! # Implementing hooks

use anyhow::Result;
use irc::client::prelude::*;

mod intensify;
mod pet;
mod shifty_eyes;

pub use intensify::*;
pub use pet::*;
pub use shifty_eyes::*;

pub mod sed;
pub mod wolfram_alpha;

/// Replies with some information about the bot
pub fn about(bot: &crate::Bot, msg: Message) -> Result<()> {
    bot.send_privmsg(
        msg.response_target().unwrap(),
        &format!(
            "{name} is {name} - https://gitlab.com/cocainefarm/gnulag/catinator",
            name = bot.config.user.nickname
        )
        .to_string(),
    )
    .unwrap();

    Ok(())
}

/// Listen to AUTHENTICATE messages and perform SASL authentication
pub fn sasl(bot: &crate::Bot, msg: Message) -> Result<()> {
    match msg.command {
        Command::AUTHENTICATE(text) => {
            use sasl::client::mechanisms::Plain;
            use sasl::client::Mechanism;
            use sasl::common::Credentials;

            if text == "+" {
                let creds = Credentials::default()
                    .with_username(bot.config.clone().user.username)
                    .with_password(bot.config.clone().user.password.unwrap());

                let mut mechanism = Plain::from_credentials(creds)?;

                let initial_data = mechanism.initial();

                bot.irc_client.send_sasl(base64::encode(initial_data))?;
                bot.irc_client.send(Command::CAP(
                    None,
                    irc_proto::command::CapSubCommand::END,
                    None,
                    None,
                ))?;
            }
        }
        _ => (),
    }

    Ok(())
}
