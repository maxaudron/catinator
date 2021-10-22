use anyhow::{Context, Error, Result};
use async_trait::async_trait;
use reqwest::{get, Url};

pub struct Isgd;

#[async_trait]
impl super::UrlShortener for Isgd {
    async fn shorten(url: &str) -> Result<String, Error> {
        Ok(get(Url::parse(&format!(
            "https://is.gd/create.php?format=simple&url={}",
            url
        ))
        .context("Failed to parse url")?)
        .await
        .context("Failed to make request")?
        .text()
        .await
        .context("failed to get request response text")?)
    }
}
