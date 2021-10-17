use std::{borrow::Cow, str::Chars};

use bitflags::bitflags;
use regex::Regex;

use crate::util::formatting::Formatting;

type Commands = Vec<Command>;

#[derive(Debug, Clone)]
pub struct Command {
    left: Regex,
    right: String,
    flags: Flags,
}

impl PartialEq for Command {
    fn eq(&self, other: &Self) -> bool {
        self.left.as_str() == other.left.as_str()
            && self.right == other.right
            && self.flags == other.flags
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ParseError {
    #[error("not a sed command, does not start with 's/'")]
    NotSedCommand,
    #[error("unknown flag")]
    InvalidFlag,
    #[error(transparent)]
    InvalidRegex(#[from] regex::Error),
}

impl Command {
    pub fn from_str(input: &str) -> Result<Command, ParseError> {
        let mut chars = input.chars();

        if chars.next().unwrap() == 's' && chars.next().unwrap() == '/' {
            let left = Command::parse_segment(&mut chars)?;
            let right = Command::parse_segment(&mut chars)?.bold();
            let flags = Flags::from_chars(&mut chars)?;

            let left = Regex::new(&format!("(?{}){}", flags.to_string(), left))
                .map_err(|err| ParseError::InvalidRegex(err))?;

            return Ok(Command { left, right, flags });
        } else {
            return Err(ParseError::NotSedCommand);
        }
    }

    pub fn from_str_multiple(input: &str) -> Result<Commands, ParseError> {
        let mut commands = Commands::new();

        let mut chars = input.chars();

        loop {
            let s = chars.next();
            let slash = chars.next();

            if s.is_some() && slash.is_some() {
                if s.unwrap() == 's' && slash.unwrap() == '/' {
                    let left = Command::parse_segment(&mut chars)?;
                    let right = Command::parse_segment(&mut chars)?.bold();
                    let flags = Flags::from_chars(&mut chars)?;

                    let left = Regex::new(&format!("(?{}){}", flags.to_string(), left))
                        .map_err(|err| ParseError::InvalidRegex(err))?;

                    commands.push(Command { left, right, flags });
                } else {
                    return Err(ParseError::NotSedCommand);
                }
            } else {
                break;
            }
        }

        Ok(commands)
    }

    fn parse_segment(chars: &mut Chars) -> Result<String, ParseError> {
        let mut last_char = '/';
        let mut output = String::new();

        while let Some(c) = chars.next() {
            if c == '/' && last_char != '\\' {
                break;
            } else if c == '/' && last_char == '\\' {
                output.pop().unwrap();
            }

            output.push(c);
            last_char = c;
        }

        Ok(output)
    }

    pub fn execute(self, target: &str) -> Cow<str> {
        let result: Cow<str>;

        if self.flags.contains(Flags::GLOBAL) {
            result = self.left.replace_all(target, self.right);
        } else {
            result = self.left.replace(target, self.right);
        }

        return result;
    }

    pub fn regex(&self) -> &Regex {
        &self.left
    }
}

bitflags! {
    /// i     case-insensitive: letters match both upper and lower case
    /// m     multi-line mode: ^ and $ match begin/end of line
    /// s     allow . to match \n
    /// U     swap the meaning of x* and x*?
    /// x     ignore whitespace and allow line comments (starting with `#`)
    struct Flags: u32 {
        const GLOBAL           = 0b00000001;
        const CASE_INSENSITIVE = 0b00000010;
        const SINGLE_LINE      = 0b00001000;
        const UNGREEDY         = 0b00010000;
        const EXTENDED         = 0b00100000;
    }
}

impl Flags {
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        result.push('m');

        if self.contains(Flags::CASE_INSENSITIVE) {
            result.push('i');
        }

        if self.contains(Flags::SINGLE_LINE) {
            result.push('s');
        }

        if self.contains(Flags::UNGREEDY) {
            result.push('U');
        }

        if self.contains(Flags::EXTENDED) {
            result.push('x');
        }

        return result;
    }

    pub fn from_chars(chars: &mut Chars) -> Result<Flags, ParseError> {
        let mut flags: Flags = Flags::empty();

        while let Some(c) = chars.next() {
            match c {
                'g' => {
                    flags = flags | Flags::GLOBAL;
                }
                'i' => {
                    flags = flags | Flags::CASE_INSENSITIVE;
                }
                's' => {
                    flags = flags | Flags::SINGLE_LINE;
                }
                'U' => {
                    flags = flags | Flags::UNGREEDY;
                }
                'x' => {
                    flags = flags | Flags::EXTENDED;
                }
                ';' => return Ok(flags),
                _ => return Err(ParseError::InvalidFlag),
            };
        }

        Ok(flags)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const COMMAND_SIMPLE: &str = "s/replace/replacee/ig";
    const COMMAND_MULTIPLE: &str = "s/replace/replacee/ig;s/two/tworeplace/i";

    #[test]
    fn test_parse_segment() -> Result<(), ParseError> {
        let mut chars = "replace/replacee/ig".chars();

        let left = "replace";
        let right = Command::parse_segment(&mut chars)?;

        assert_eq!(left, right);

        let left = "replacee";
        let right = Command::parse_segment(&mut chars)?;

        assert_eq!(left, right);

        Ok(())
    }

    #[test]
    fn test_flags_from_chars() -> Result<(), ParseError> {
        let mut chars = "ig".chars();

        let left = Flags::CASE_INSENSITIVE | Flags::GLOBAL;
        let right = Flags::from_chars(&mut chars)?;

        assert_eq!(left, right);

        let mut chars = "igf".chars();
        let right = Flags::from_chars(&mut chars);

        assert_eq!(Err(ParseError::InvalidFlag), right);

        Ok(())
    }

    #[test]
    fn test_flags_from_chars_with_terminator() -> Result<(), ParseError> {
        let mut chars = "ig;bla".chars();

        let left = Flags::CASE_INSENSITIVE | Flags::GLOBAL;
        let right = Flags::from_chars(&mut chars)?;

        assert_eq!(left, right);
        assert_eq!("bla", chars.as_str());

        Ok(())
    }

    #[test]
    fn test_new_command_simple() -> Result<(), ParseError> {
        let left = Command::from_str(COMMAND_SIMPLE)?;
        let right = Command {
            left: Regex::new("(?mi)replace").unwrap(),
            right: "\x02replacee\x02".to_string(),
            flags: Flags::CASE_INSENSITIVE | Flags::GLOBAL,
        };

        assert_eq!(left, right);

        Ok(())
    }

    #[test]
    fn test_new_command_simple_escaped_slash() -> Result<(), ParseError> {
        let left = Command::from_str(r#"s/repl\/ace/replacee"#)?;
        let right = Command {
            left: Regex::new("(?m)repl/ace").unwrap(),
            right: "\x02replacee\x02".to_string(),
            flags: Flags::empty(),
        };

        assert_eq!(left, right);

        Ok(())
    }

    #[test]
    fn test_new_command_simple_no_terminating_slash() -> Result<(), ParseError> {
        let left = Command::from_str("s/replace/replacee")?;
        let right = Command {
            left: Regex::new("(?m)replace").unwrap(),
            right: "\x02replacee\x02".to_string(),
            flags: Flags::empty(),
        };

        assert_eq!(left, right);

        Ok(())
    }

    #[test]
    fn test_new_command_complex_regex() -> Result<(), ParseError> {
        let left =
            Command::from_str(r#"s/http(?:s?):\/\/regex101\.com\/r\/([a-zA-Z0-9]{1,6})?$/$1/g"#)?;
        let right = Command {
            left: Regex::new(r#"(?m)http(?:s?)://regex101\.com/r/([a-zA-Z0-9]{1,6})?$"#).unwrap(),
            right: "\x02$1\x02".to_string(),
            flags: Flags::GLOBAL,
        };

        assert_eq!(left, right);

        Ok(())
    }

    #[test]
    fn test_new_command_multiple_fail() -> Result<(), ParseError> {
        let left = Command::from_str_multiple(COMMAND_SIMPLE)?;
        let right = vec![Command {
            left: Regex::new("(?mi)replace").unwrap(),
            right: "\x02replacee\x02".to_string(),
            flags: Flags::CASE_INSENSITIVE | Flags::GLOBAL,
        }];

        assert_eq!(left, right);

        Ok(())
    }

    #[test]
    fn test_new_command_multiple() -> Result<(), ParseError> {
        let left = Command::from_str_multiple(COMMAND_MULTIPLE)?;
        let right = vec![
            Command {
                left: Regex::new("(?mi)replace").unwrap(),
                right: "\x02replacee\x02".to_string(),
                flags: Flags::CASE_INSENSITIVE | Flags::GLOBAL,
            },
            Command {
                left: Regex::new("(?mi)two").unwrap(),
                right: "\x02tworeplace\x02".to_string(),
                flags: Flags::CASE_INSENSITIVE,
            },
        ];

        assert_eq!(left, right);

        Ok(())
    }

    #[test]
    fn test_run_regex() -> Result<(), ParseError> {
        let cmd = Command::from_str(COMMAND_SIMPLE)?;

        let left = "this is a sentence to \x02replacee\x02 text in";
        let right = cmd.execute("this is a sentence to replace text in");

        assert_eq!(left, right);

        Ok(())
    }
}
