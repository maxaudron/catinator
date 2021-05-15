use anyhow::{anyhow, Result};
use irc::client::prelude::*;

use sedregex::ReplaceCommand;

use std::cell::RefCell;

static LOG_MAX_SIZE: usize = 10000;

thread_local!(static LOG: RefCell<Vec::<(String, String)>> = RefCell::new(Vec::with_capacity(LOG_MAX_SIZE)));
thread_local!(static RE: regex::Regex = regex::Regex::new(r"^s/").unwrap());

pub fn log(_bot: &crate::Bot, msg: Message) -> Result<()> {
    log_msg(msg)
}

fn log_msg(msg: Message) -> Result<()> {
    if let Command::PRIVMSG(_, text) = msg.command.clone() {
        LOG.with(|log_cell| {
            let mut log = log_cell.borrow_mut();
            if log.len() >= LOG_MAX_SIZE {
                let _ = log.pop();
            }
            log.push((msg.source_nickname().unwrap().to_string(), text))
        });
    }
    Ok(())
}

pub fn replace(bot: &crate::Bot, msg: Message) -> Result<()> {
    match find_and_replace(&msg) {
        Ok(res) => {
            bot.send_privmsg(msg.response_target().unwrap(), res.as_str())
                .unwrap();
            Ok(())
        }
        Err(_) => Ok(()),
    }
}

fn find_and_replace(msg: &Message) -> Result<String> {
    if let Command::PRIVMSG(_, text) = msg.command.clone() {
        let cmd = match ReplaceCommand::new(text.as_str()) {
            Ok(cmd) => cmd,
            Err(_) => return Err(anyhow!("building replace command failed")),
        };

        return LOG.with(|log_cell| {
            log_cell
                .borrow()
                .iter()
                .rev()
                .find(|(_, text)| cmd.expr.is_match(text) && !RE.with(|re| re.is_match(text)))
                .and_then(|(nick, text)| Some(format!("<{}> {}", nick, cmd.execute(text))))
                .map_or(Err(anyhow!("replace failed")), |v| Ok(v))
        });
    }

    Err(anyhow!("not a privmsg"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    fn populate_log() {
        LOG.with(|log_cell| {
            let mut log = log_cell.borrow_mut();
            log.push((
                "user".to_string(),
                "this is a long message which will be replaced".to_string(),
            ));
            for _ in 0..LOG_MAX_SIZE-1 {
                log.push((
                    "user".to_string(),
                    "this is a long message which doesn't matter".to_string(),
                ))
            }
        });
    }

    #[test]
    fn test_log_limit() {
        populate_log();

        LOG.with(|log_cell| {
            let log = log_cell.borrow();
            assert_eq!(log.len(), LOG_MAX_SIZE)
        });

        log_msg(Message {
            tags: None,
            prefix: Some(Prefix::Nickname(
                "user".to_string(),
                "username".to_string(),
                "userhost".to_string(),
            )),
            command: Command::PRIVMSG(
                "#channel".to_string(),
                "this is the 10001th message".to_string(),
            ),
        })
        .unwrap();

        LOG.with(|log_cell| {
            let log = log_cell.borrow();
            assert_eq!(log.len(), LOG_MAX_SIZE)
        });
    }

    #[test]
    fn test_replace() {
        populate_log();
        assert_eq!(
            find_and_replace(&Message {
                tags: None,
                prefix: None,
                command: Command::PRIVMSG(
                    "#channel".to_string(),
                    "s/will be/has been/".to_string(),
                ),
            })
            .unwrap(),
            "<user> this is a long message which has been replaced"
        )
    }

    #[test]
    fn test_replace_complex() {
        populate_log();
        assert_eq!(
            find_and_replace(&Message {
                tags: None,
                prefix: None,
                command: Command::PRIVMSG(
                    "#channel".to_string(),
                    "s/(will).*(be)/$2 $1/".to_string(),
                ),
            })
            .unwrap(),
            "<user> this is a long message which be will replaced"
        )
    }

    #[bench]
    fn bench_replace(b: &mut Bencher) {
        populate_log();
        b.iter(|| {
            find_and_replace(&Message {
                tags: None,
                prefix: None,
                command: Command::PRIVMSG(
                    "#channel".to_string(),
                    "s/will be/has been/".to_string(),
                ),
            })
        });
    }

    #[bench]
    fn bench_replace_complex(b: &mut Bencher) {
        populate_log();
        b.iter(|| {
            find_and_replace(&Message {
                tags: None,
                prefix: None,
                command: Command::PRIVMSG(
                    "#channel".to_string(),
                    "s/will be/has been/".to_string(),
                ),
            })
        });
    }
}
