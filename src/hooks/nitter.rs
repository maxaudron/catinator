use std::sync::LazyLock;

use anyhow::{Context, Result};
use irc::client::prelude::*;
use regex::Regex;

static RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"https:\/\/(?:twitter|x)\.com\/(\S+?)(?:\s|$)").unwrap());

const URL: &str = "https://xcancel.com/";

pub fn nitter(bot: &crate::Bot, msg: Message) -> Result<()> {
    if let Command::PRIVMSG(_, text) = msg.command.clone() {
        let path = RE
            .captures(&text)
            .context("failed to capture twitter url path")?
            .get(1)
            .context("failed to get path capture group")?
            .as_str();

        bot.send_privmsg(
            msg.response_target()
                .context("failed to get response target")?,
            format!("get cancled {URL}{path}").as_str(),
        )?;
    }

    Ok(())
}
