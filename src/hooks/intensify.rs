use anyhow::Result;
use irc::client::prelude::*;
use macros::privmsg;

pub fn intensify(bot: &crate::Bot, msg: Message) -> Result<()> {
    privmsg!(msg, {
        let mut chars = text.chars();
        chars.next();
        chars.next_back();
        let content = chars.as_str();

        bot.send_privmsg(
            msg.response_target().unwrap(),
            format!("\x02\x0304[\x1d{} INTENSIFIES\x1d]\x03\x0F", content.to_uppercase()).as_str(),
        )?;
    })
}
