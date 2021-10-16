use anyhow::{Context, Error, Result};
use async_trait::async_trait;
use reqwest::{get, Url};
use urlparse::quote_plus as urlparse_quote_plus;

#[async_trait]
pub trait UrlShortener {
    fn new() -> Self;
    async fn shorten(&self, url: &str) -> Result<String, Error>;
}

pub struct IsgdUrlShortener {}

#[async_trait]
impl UrlShortener for IsgdUrlShortener {
    fn new() -> Self {
        Self {}
    }

    async fn shorten(&self, url: &str) -> Result<String, Error> {
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

pub fn quote_plus(text: &str) -> Result<String, Error> {
    Ok(urlparse_quote_plus(text, b"")?)
}

#[cfg(test)]
mod tests {
    use super::quote_plus;
    use anyhow::{Error, Result};

    #[test]
    fn test_quote_plus_1() -> Result<(), Error> {
        assert_eq!(quote_plus("5/10")?, "5%2F10");
        Ok(())
    }

    #[test]
    fn test_quote_plus_2() -> Result<(), Error> {
        assert_eq!(quote_plus("1 * 2")?, "1+%2A+2");
        Ok(())
    }

    #[test]
    fn test_quote_plus_3() -> Result<(), Error> {
        assert_eq!(quote_plus(&quote_plus("1 * 2")?)?, "1%2B%252A%2B2");
        Ok(())
    }
}
