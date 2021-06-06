use std::str;

use anyhow::Result;
use irc::client::prelude::*;
use macros::privmsg;

use rand::seq::SliceRandom;
use rand::thread_rng;

const PET_RESPONSE: [&str; 5] = [
    "purrs",
    "moews loudly",
    "walks away",
    "snuggles back",
    "strikes you with it's sharp claws",
];

/// Pet cat
///
/// Sends some random action when petted.
///
/// # See also
///
/// - [`Bot::send_action`]
/// - RESPONSE
pub fn pet(bot: &crate::Bot, msg: Message) -> Result<()> {
    privmsg!(msg, {
        bot.send_action(
            msg.response_target().unwrap(),
            PET_RESPONSE.choose(&mut thread_rng()).unwrap(),
        )?;
    })
}
