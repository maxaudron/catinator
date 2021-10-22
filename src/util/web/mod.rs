use anyhow::{Error, Result};
use async_trait::async_trait;
use urlparse::quote_plus as urlparse_quote_plus;

pub mod url_shorteners;

/// Shorten urls
#[async_trait]
pub trait UrlShortener {
    /// Call this method with the url you want shortened.
    /// Returns the shortened url.
    async fn shorten(url: &str) -> Result<String, Error>;
}

/// quote strings to be URL save
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
