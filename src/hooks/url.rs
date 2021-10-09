use anyhow::{bail, Context, Error, Result};
use irc::client::prelude::*;

use regex::Regex;

extern crate kuchiki;
use kuchiki::{parse_html, traits::*};
use reqwest::{get, Url};
use tracing::trace;

pub const URL_REGEX: &str = r#"(https?://|www.)\S+"#;

#[tracing::instrument]
pub fn url_parser(msg: &str) -> Vec<String> {
    let url_regex = Regex::new(URL_REGEX).unwrap();

    url_regex
        .find_iter(msg)
        .map(|u| u.as_str().to_string().replace("www.", "https://"))
        .collect::<Vec<String>>()
}

#[tracing::instrument]
pub async fn url_title(url: &str) -> Result<String, Error> {
    let body = get(Url::parse(url).context("Failed to parse url")?)
        .await
        .context("Failed to make request")?
        .text()
        .await
        .context("failed to get request response text")?;

    let document = parse_html().one(body);
    match document.select("title") {
        Ok(title) => Ok(title
            .into_iter()
            .nth(0)
            .context("title did not have text")?
            .text_contents()),
        Err(_) => bail!("could not find title"),
    }
}

#[tracing::instrument(skip(bot))]
pub async fn url_preview(bot: &crate::Bot, msg: Message) -> Result<()> {
    if let Command::PRIVMSG(target, text) = msg.command.clone() {
        let mut futures: Vec<tokio::task::JoinHandle<_>> = Vec::new();

        for url in url_parser(&text) {
            futures.push(tokio::spawn(async move {
                trace!("got url: {:?}", url);
                match url_title(&url.as_str()).await {
                    Ok(title) => {
                        trace!("extracted title from url: {:?}, {:?}", title, url);
                        Ok(title)
                    }
                    Err(err) => bail!("Failed to get urls title: {:?}", err),
                }
            }))
        }

        let titles = futures::future::join_all(futures).await;

        let titles: Vec<String> = titles
            .into_iter()
            .filter_map(|x| x.ok())
            .filter_map(|x| x.ok())
            .collect();

        if !titles.is_empty() {
            bot.send_privmsg(&target, &msg_builder(&titles))?;
        }
    }
    Ok(())
}

#[tracing::instrument]
pub fn msg_builder(titles: &Vec<String>) -> String {
    format!(
        "Title{}: {}",
        if titles.len() > 1 { "s" } else { "" },
        titles.join(" --- ")
    )
}

#[cfg(test)]
mod tests {

    use super::msg_builder;
    use super::url_parser;
    use super::url_title;
    use anyhow::{Error, Result};

    #[test]
    fn test_url_titel() {
        let title: String =
            tokio_test::block_on(url_title("https://news.ycombinator.com")).unwrap();
        assert_eq!(title.as_str(), "Hacker News");

        let title: String = tokio_test::block_on(url_title("https://google.com")).unwrap();
        assert_eq!(title.as_str(), "Google");

        assert!(tokio_test::block_on(url_title("random_site")).is_err())
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

    #[test]
    fn test_msg_builder() {
        let msg = msg_builder(&Vec::from(["hello".to_string(), "world".to_string()]));
        assert_eq!("Titles: hello --- world", msg);

        let msg = msg_builder(&Vec::from(["hello".to_string()]));
        assert_eq!("Title: hello", msg);
    }

    #[test]
    fn test_all() {
        let mut titles: Vec<String> = Vec::new();
        let text = "https://news.ycombinator.com www.google.com https://youtube.com";
        let urls = url_parser(&text);

        assert_eq!(urls.len(), 3);

        for url in &urls {
            if let Ok(title) = tokio_test::block_on(url_title(&url.as_str())) {
                titles.push(title);
            }
        }
        assert_eq!(
            msg_builder(&titles),
            "Titles: Hacker News --- Google --- YouTube"
        );
    }
}
