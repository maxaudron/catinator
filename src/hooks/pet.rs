use anyhow::{Context, Result};
use irc::client::prelude::*;
use macros::privmsg;

use rand::{prelude::IndexedRandom, rng};

const PET_RESPONSE: [&str; 5] = [
    "purrs",
    "meows loudly",
    "walks away",
    "snuggles back",
    "strikes you with it's sharp claws",
];

/// Pet the cat, get rekt
///
/// Sends some random action when petted.
pub fn pet(bot: &crate::Bot, msg: Message) -> Result<()> {
    privmsg!(msg, {
        bot.send_action(
            msg.response_target()
                .context("failed to get response target")?,
            PET_RESPONSE
                .choose(&mut rng())
                .context("failed choosing a pet response")?,
        )?;
    })
}
