use anyhow::{Context, Result, anyhow, bail};
use irc::client::prelude::*;

use sedregex::ReplaceCommand;

use std::collections::HashMap;

static LOG_MAX_SIZE: usize = 10000;

thread_local!(static RE: regex::Regex = regex::Regex::new(r"^s/").unwrap());

pub struct Sed(HashMap<String, Vec<(String, String)>>);

impl Sed {
    pub fn new() -> Sed {
        Sed(HashMap::new())
    }

    pub fn log(&mut self, _bot: &crate::Bot, msg: Message) -> Result<()> {
        self.log_msg(msg).context("failed to log new message")
    }

    fn log_msg(&mut self, msg: Message) -> Result<()> {
        if let Command::PRIVMSG(target, mut text) = msg.command.clone() {
            if text.starts_with("\x01ACTION") {
                text = text.replace("\x01ACTION", "\x01\x01");
            }

            match self.0.get_mut(&target) {
                Some(log) => {
                    if log.len() >= LOG_MAX_SIZE {
                        let _ = log.remove(0);
                    }
                    log.push((msg.source_nickname().unwrap().to_string(), text))
                }
                None => {
                    let mut log = Vec::with_capacity(LOG_MAX_SIZE);
                    log.push((msg.source_nickname().unwrap().to_string(), text));
                    self.0.insert(target, log);
                }
            }
        }
        Ok(())
    }

    pub fn replace(&mut self, bot: &crate::Bot, msg: Message) -> Result<()> {
        match self.find_and_replace(&msg) {
            Ok(res) => match bot.send_privmsg(msg.response_target().unwrap(), res.as_str()) {
                Ok(_) => Ok(()),
                Err(_) => bail!(
                    "failed to send message: \"{:?}\" to channel: {:?}",
                    msg.response_target().unwrap(),
                    res
                ),
            },
            Err(_) => bail!("did not find match for: {:?}", msg),
        }
    }

    fn find_and_replace(&mut self, msg: &Message) -> Result<String> {
        if let Command::PRIVMSG(target, text) = msg.command.clone() {
            let cmd = match ReplaceCommand::new(text.as_str()) {
                Ok(cmd) => cmd,
                Err(_) => return Err(anyhow!("building replace command failed")),
            };

            let log = self
                .0
                .get(&target)
                .context("did not find log for current channel")?;

            return log
                .iter()
                .rev()
                .find(|(_, text)| cmd.expr.is_match(text) && !RE.with(|re| re.is_match(text)))
                .and_then(|(nick, text)| {
                    if text.starts_with("\x01\x01") {
                        Some(format!(
                            "* {}{}",
                            nick,
                            cmd.execute(text.replace("\x01", ""))
                        ))
                    } else {
                        Some(format!("<{}> {}", nick, cmd.execute(text)))
                    }
                })
                .map_or(Err(anyhow!("replace failed")), |v| Ok(v));
        }

        Err(anyhow!("not a privmsg"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn populate_log() -> Sed {
        let mut sed = Sed::new();

        sed.log_msg(
            Message::new(
                Some("user!user@user.com"),
                "PRIVMSG",
                vec!["user", "this is a long message which will be replaced"],
            )
            .unwrap(),
        )
        .unwrap();

        for _ in 0..LOG_MAX_SIZE - 1 {
            sed.log_msg(
                Message::new(
                    Some("user!user@user.com"),
                    "PRIVMSG",
                    vec!["user", "this is a long message which doesn't matter"],
                )
                .unwrap(),
            )
            .unwrap();
        }

        return sed;
    }

    #[test]
    fn test_log_push_max() {
        let mut sed = Sed::new();

        sed.log_msg(
            Message::new(Some("user!user@user.com"), "PRIVMSG", vec!["user", "one"]).unwrap(),
        )
        .unwrap();

        for _ in 0..LOG_MAX_SIZE - 2 {
            sed.log_msg(
                Message::new(Some("user!user@user.com"), "PRIVMSG", vec!["user", "two"]).unwrap(),
            )
            .unwrap();
        }
        sed.log_msg(
            Message::new(Some("user!user@user.com"), "PRIVMSG", vec!["user", "three"]).unwrap(),
        )
        .unwrap();

        {
            let log = sed.0.get("user").unwrap();
            assert_eq!(
                log[LOG_MAX_SIZE - 1],
                ("user".to_string(), "three".to_string())
            );
            assert_eq!(log[0], ("user".to_string(), "one".to_string()));
        }

        sed.log_msg(
            Message::new(Some("user!user@user.com"), "PRIVMSG", vec!["user", "four"]).unwrap(),
        )
        .unwrap();

        {
            let log = sed.0.get("user").unwrap();

            assert_eq!(
                log[LOG_MAX_SIZE - 1],
                ("user".to_string(), "four".to_string())
            );
            assert_eq!(log[0], ("user".to_string(), "two".to_string()));
        }
    }

    #[test]
    fn test_log_limit() {
        let mut sed = populate_log();

        {
            let log = sed.0.get("user").unwrap();
            assert_eq!(log.len(), LOG_MAX_SIZE);
        }

        sed.log_msg(
            Message::new(
                Some("user!user@user.com"),
                "PRIVMSG",
                vec!["user", "this is the 10001th message"],
            )
            .unwrap(),
        )
        .unwrap();

        {
            let log = sed.0.get("user").unwrap();
            assert_eq!(log.len(), LOG_MAX_SIZE);
        }
    }

    #[test]
    fn test_replace() {
        let mut sed = populate_log();
        assert_eq!(
            sed.find_and_replace(&Message {
                tags: None,
                prefix: None,
                command: Command::PRIVMSG("user".to_string(), "s/will be/has been/".to_string(),),
            })
            .unwrap(),
            "<user> this is a long message which has been replaced"
        )
    }

    #[test]
    fn test_replace_complex() {
        let mut sed = populate_log();
        assert_eq!(
            sed.find_and_replace(&Message {
                tags: None,
                prefix: None,
                command: Command::PRIVMSG("user".to_string(), "s/(will).*(be)/$2 $1/".to_string(),),
            })
            .unwrap(),
            "<user> this is a long message which be will replaced"
        )
    }
}

#[cfg(all(test, feature = "bench"))]
mod bench {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_replace(b: &mut Bencher) {
        let mut sed = tests::populate_log();
        b.iter(|| {
            sed.find_and_replace(&Message {
                tags: None,
                prefix: None,
                command: Command::PRIVMSG("user".to_string(), "s/will be/has been/".to_string()),
            })
        });
    }

    #[bench]
    fn bench_replace_complex(b: &mut Bencher) {
        let mut sed = tests::populate_log();
        b.iter(|| {
            sed.find_and_replace(&Message {
                tags: None,
                prefix: None,
                command: Command::PRIVMSG("user".to_string(), "s/(will).*(be)/$2 $1/".to_string()),
            })
        });
    }
}
