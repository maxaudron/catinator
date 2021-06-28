use anyhow::{anyhow, Result};
use irc::client::prelude::*;

const EYES: [char; 11] = ['^', 'v', 'V', '>', '<', 'x', 'X', '-', 'o', 'O', '.'];
const NOSE: [char; 7] = ['.', '_', '-', ';', '\'', '"', '~'];

pub fn shifty_eyes(bot: &crate::Bot, msg: Message) -> Result<()> {
    if let Command::PRIVMSG(_, text) = msg.command.clone() {
        if text.len() == 3 {
            let mut chars = text.chars();
            let mut left = chars.next().unwrap();
            let middle = chars.next().unwrap();
            let mut right = chars.next().unwrap();

            if EYES.contains(&left) && NOSE.contains(&middle) && EYES.contains(&right) {
                left = invert(left)?;
                right = invert(right)?;

                let mut result = String::new();
                result.push(left);
                result.push(middle);
                result.push(right);

                bot.send_privmsg(msg.response_target().unwrap(), result.as_str())?;
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
        'X' => Ok('O'),
        '-' => Ok('o'),
        'o' => Ok('-'),
        'O' => Ok('-'),
        _ => Err(anyhow!("not a valid char")),
    }
}
