use anyhow::{Context, Error, Result};
use reqwest::{get, Url};

// TODO: Either catinator should have a URL shortening utility module,
// or we should start our own service
pub(crate) async fn shorten_url(url: &str) -> Result<String, Error> {
    // This just uses the first service gonzobot uses too
    let short_url = get(Url::parse(&format!(
        "https://is.gd/create.php?format=simple&url={}",
        url
    ))
    .context("Failed to parse url")?)
    .await
    .context("Failed to make request")?
    .text()
    .await
    .context("failed to get request response text")?;

    Ok(short_url)
}
