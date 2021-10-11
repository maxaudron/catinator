use crate::util::web::shorten_url;
use anyhow::{bail, Context, Error, Result};
use futures::try_join;
use irc::client::prelude::*;
use macros::privmsg;
use reqwest::{get, Url};
use serde::{Deserialize, Serialize};
use serde_json::Result as SerdeJsonResult;

#[derive(Serialize, Deserialize, Debug)]
struct WaResponse {
    queryresult: QueryResult,
}

#[derive(Serialize, Deserialize, Debug)]
struct QueryResult {
    pods: Vec<Pod>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Pod {
    title: String,
    id: String,
    primary: Option<bool>,
    subpods: Vec<SubPod>,
}

#[derive(Serialize, Deserialize, Debug)]
struct SubPod {
    plaintext: String,
}

fn parse_json(str_data: &str) -> SerdeJsonResult<WaResponse> {
    let w: WaResponse = serde_json::from_str(str_data)?;
    Ok(w)
}

/// Reduces all 'pod' plaintexts to a single string.
/// Same as gonzobot does it.
fn to_single_string(wa_res: WaResponse) -> String {
    wa_res
        .queryresult
        .pods
        .iter()
        .filter(|it| it.id.to_lowercase() != "input" && it.primary.is_some())
        .map(|pod| {
            let subpod_texts = pod
                .subpods
                .iter()
                .map(|subpod| subpod.plaintext.clone())
                .collect::<Vec<String>>()
                .join(", ");

            format!("{}: {}", &pod.title, subpod_texts)
        })
        .collect::<Vec<String>>()
        .join(" - ")
}

fn get_url(query_str: &str, api_key: Option<&str>, base_url: Option<&str>) -> String {
    let wa_url = "http://api.wolframalpha.com";
    let api_url = format!(
        "{}/v2/query?input={}&appid={}&output=json",
        base_url.unwrap_or(wa_url),
        query_str,
        api_key.unwrap_or("XXX"), // Allow tests to run without a key
    );
    api_url
}

async fn send_wa_req(url: &str) -> Result<String, Error> {
    let body = get(Url::parse(url).context("Failed to parse url")?)
        .await
        .context("Failed to make request")?
        .text()
        .await
        .context("failed to get request response text")?;
    Ok(body)
}

async fn handle_wa_req(url: &str) -> Result<WaResponse, Error> {
    let res_body = send_wa_req(url).await?;
    let parsed = parse_json(&res_body)?;
    Ok(parsed)
}

/// Sends a request to the Wolfram Alpha API, returns a plain text response.
#[tracing::instrument]
async fn wa_query(
    query_str: &str,
    api_key: Option<&str>,
    base_url: Option<&str>,
) -> Result<String, Error> {
    let user_url = format!("http://www.wolframalpha.com/input/?i={}", query_str);
    let user_url_shortened_fut = shorten_url(&user_url);

    let url = get_url(query_str, api_key, base_url);
    let wa_res_fut = handle_wa_req(&url);

    // Can't just (foo.await, bar.await), smh
    // https://rust-lang.github.io/async-book/06_multiple_futures/02_join.html
    let (wa_res, user_url_shortened) = try_join!(wa_res_fut, user_url_shortened_fut)?;

    Ok(format!(
        "{} - {}",
        &user_url_shortened,
        to_single_string(wa_res)
    ))
}

pub async fn wa(bot: &crate::Bot, msg: Message) -> Result<()> {
    privmsg!(msg, {
        let content = text.chars().as_str().splitn(2, " ").collect::<Vec<&str>>()[1];
        if content.is_empty() {
            bail!("Empty input for WA query");
        }
        bot.send_privmsg(
            msg.response_target()
                .context("failed to get response target")?,
            &wa_query(content, Some(&bot.config.settings.wa_api_key), None).await?,
        )?;
    })
}

#[cfg(test)]
mod tests {

    use super::wa_query;
    use anyhow::{Error, Result};
    use mockito::{self, Matcher};

    #[tokio::test]
    async fn test_query_result_json_parsing() -> Result<(), Error> {
        let body = include_str!("../../tests/resources/wolfram_alpha_api_response.json");
        let _m = mockito::mock("GET", Matcher::Any)
            // Trimmed down version of a full WA response:
            .with_body(body)
            .create();
        mockito::start();

        let res = wa_query("5/10", None, Some(&mockito::server_url())).await?;
        let res_without_link = res.splitn(2, "-").collect::<Vec<&str>>()[1].trim();
        assert_eq!(res_without_link, "Exact result: 1/2 - Decimal form: 0.5");
        Ok(())
    }
}
