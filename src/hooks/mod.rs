use anyhow::Result;
use irc::client::prelude::*;

pub mod sed;
pub mod shifty_eyes;

pub use sed::*;
pub use shifty_eyes::shifty_eyes;

pub fn sasl(bot: &crate::Bot, msg: Message) -> Result<()> {
    match msg.command {
        Command::AUTHENTICATE(text) => {
            use sasl::client::mechanisms::Plain;
            use sasl::client::Mechanism;
            use sasl::common::Credentials;

            if text == "+" {
                let creds = Credentials::default()
                    .with_username(bot.config.clone().user.username)
                    .with_password(bot.config.clone().user.password);

                let mut mechanism = Plain::from_credentials(creds).unwrap();

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
