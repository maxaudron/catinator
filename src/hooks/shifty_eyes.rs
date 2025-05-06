use anyhow::{bail, Context, Result};
use irc::client::prelude::*;

const EYES: [char; 7] = ['^', 'v', 'V', '>', '<', 'x', 'X'];
const NOSE: [char; 7] = ['.', '_', '-', ';', '\'', '"', '~'];

/// you are being watched <.<
pub fn shifty_eyes(bot: &crate::Bot, msg: Message) -> Result<()> {
    if let Command::PRIVMSG(_, text) = msg.command.clone() {
        if text.len() == 3 {
            let mut chars = text.chars();
            let mut left = chars.next().context("failed to get next character")?;
            let middle = chars.next().context("failed to get next character")?;
            let mut right = chars.next().context("failed to get next character")?;

            if EYES.contains(&left) && NOSE.contains(&middle) && EYES.contains(&right) {
                left = invert(left)?;
                right = invert(right)?;

                let mut result = String::new();
                result.push(left);
                result.push(middle);
                result.push(right);

                bot.send_privmsg(
                    msg.response_target()
                        .context("failed to get response target")?,
                    result.as_str(),
                )?;
            }
        }
    }

    Ok(())
}

fn invert(input: char) -> Result<char> {
    match input {
        '^' => Ok('v'),
        'v' => Ok('^'),
        'V' => Ok('^'),
        '>' => Ok('<'),
        '<' => Ok('>'),
        'x' => Ok('o'),
        '.' => Ok('o'),
        'X' => Ok('O'),
        '-' => Ok('o'),
        'o' => Ok('-'),
        'O' => Ok('-'),
        _ => bail!("not a valid char"),
    }
}
