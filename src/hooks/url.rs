use anyhow::Result;
use irc::client::prelude::*;

use regex::Regex;

extern crate kuchiki;
use kuchiki::traits::*;

pub const URL_REGEX: &str = r#"(https?://|www.)\S+"#;

pub fn url_parser(msg: &str) -> Vec<String> {
    let url_regex = Regex::new(URL_REGEX).unwrap();

    url_regex
        .find_iter(msg)
        .map(|mat| mat.as_str().to_string())
        .collect::<Vec<String>>()
}

pub async fn url_title(url: &str) -> Option<String> {
    let body = reqwest::get(url).await.ok()?.text().await.ok()?;

    let document = kuchiki::parse_html().one(body);
    match document.select("title") {
        Ok(title) => Some(title.into_iter().nth(0)?.text_contents()),
        Err(_) => None,
    }
}

pub fn url_preview(bot: &crate::Bot, msg: Message) -> Result<()> {
    if let Command::PRIVMSG(target, text) = msg.command.clone() {
        for url in url_parser(&text) {
            if let Some(title) = futures::executor::block_on(url_title(&url.as_str())) {
                bot.send_privmsg(&target, title.as_str())?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::hooks::url::url_parser;
    use crate::hooks::url::url_title;

    #[test]
    fn test_url_titel() {
        let title: String =
            tokio_test::block_on(url_title("https://news.ycombinator.com")).unwrap();
        assert_eq!(title.as_str(), "Hacker News");

        let title: String =
            tokio_test::block_on(url_title("https://google.com")).unwrap();
        assert_eq!(title.as_str(), "Google");

        let title: Option<String> =
            tokio_test::block_on(url_title("random_site"));
        assert_eq!(title, None)

    }
    #[test]
    fn test_url_parser() {
        let url = url_parser("some message https://news.ycombinator.com/ here");
        assert_eq!(url[0], "https://news.ycombinator.com/");

        let url = url_parser("no url here!");
        assert!(url.is_empty());

        let url = url_parser(
            &[
                "https://new.ycombinator.com/ ",
                "http://news.ycombinator.com/ ",
                "www.google.com",
            ]
            .concat(),
        );
        assert_eq!(url.len(), 3);
    }
}
