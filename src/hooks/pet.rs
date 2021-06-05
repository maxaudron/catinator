use std::str;

use anyhow::Result;
use irc::client::prelude::*;
use macros::privmsg;

use rand::thread_rng;
use rand::seq::SliceRandom;

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
    let mut rng = thread_rng();
    let choice = PET_RESPONSE.choose(&mut rng);

    privmsg!(msg, {
        bot.send_action(msg.response_target().unwrap(), choice.unwrap())?;
    })
}
