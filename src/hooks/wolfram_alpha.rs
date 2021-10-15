use crate::util::{
    formatting::truncate,
    web::{quote_plus, IsgdUrlShortener, UrlShortener},
};
use anyhow::{bail, Context, Error, Result};
use futures::join;
use irc::client::prelude::*;
use macros::privmsg;
use reqwest::{get, Url};
use serde::{Deserialize, Serialize};

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

fn clean_result_text(text: &str) -> String {
    text
        // Remove newlines
        .replace("\n", "; ")
        // Remove multiple whitespace
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
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
                .map(|subpod| clean_result_text(&subpod.plaintext))
                .collect::<Vec<String>>()
                .join(", ");

            format!("{}: {}", &pod.title, subpod_texts)
        })
        .collect::<Vec<String>>()
        .join(" - ")
}

fn get_wa_api_url(
    query_str: &str,
    api_key: Option<&str>,
    base_url: Option<&str>,
) -> Result<Url, Error> {
    let wa_url = "http://api.wolframalpha.com";
    let api_url = format!(
        "{}/v2/query?input={}&appid={}&output=json",
        base_url.unwrap_or(wa_url),
        quote_plus(query_str)?,
        api_key.unwrap_or("XXX"), // Allow tests to run without a key
    );
    Url::parse(&api_url).context("Failed to parse URL")
}

async fn send_wa_req(url: &Url) -> Result<String, Error> {
    let body = get(url.to_owned())
        .await
        .context("Failed to make request")?
        .text()
        .await
        .context("failed to get request response text")?;
    Ok(body)
}

async fn handle_wa_req(url: &Url) -> Result<WaResponse, Error> {
    let res_body = send_wa_req(url).await?;
    let parsed = serde_json::from_str(&res_body)?;
    Ok(parsed)
}

/// Gets a URL users can click, leading to the main WA page.
async fn get_wa_user_short_url(input: &str) -> Result<String, Error> {
    let user_url = format!(
        "http://www.wolframalpha.com/input/?i={}",
        // For some reason some inputs need double quote calls, e.g. '1 * 2'.
        // Maybe only with is.gd though.
        quote_plus(&quote_plus(input)?)?
    );
    IsgdUrlShortener::new().shorten(&user_url).await
}

/// Sends a request to the Wolfram Alpha API, returns a plain text response.
#[tracing::instrument]
async fn wa_query(
    query_str: &str,
    api_key: Option<&str>,
    base_url: Option<&str>,
) -> Result<String, Error> {
    let user_url_shortened_fut = get_wa_user_short_url(query_str);
    let url = get_wa_api_url(query_str, api_key, base_url)?;
    let wa_res_fut = handle_wa_req(&url);

    let futs = join!(wa_res_fut, user_url_shortened_fut);
    let wa_res = match futs.0 {
        Ok(x) => x,
        // Return early if there are no results at all
        _ => return Ok("No results.".to_string()),
    };
    let user_url_shortened = futs.1?;

    let string_result = match to_single_string(wa_res) {
        // Return with user link, but no plaintext results
        x if x.is_empty() => "No plaintext results.".to_string(),
        x => x,
    };

    Ok(format!(
        "{} - {}",
        truncate(&string_result, 250), // Same length as in gonzobot
        &user_url_shortened,
    ))
}

fn get_input_query(text: &str) -> Result<String, Error> {
    let input = text.chars().as_str().splitn(2, " ").collect::<Vec<&str>>();
    if input.len() != 2 {
        bail!("Empty input for WA query");
    }
    let content = input[1].trim();
    Ok(content.to_string())
}

pub async fn wa(bot: &crate::Bot, msg: Message) -> Result<()> {
    privmsg!(msg, {
        let content = get_input_query(text)?;
        bot.send_privmsg(
            msg.response_target()
                .context("failed to get response target")?,
            &wa_query(&content, Some(&bot.config.settings.wa_api_key), None).await?,
        )?;
    })
}

#[cfg(test)]
mod tests {

    use crate::hooks::wolfram_alpha::clean_result_text;

    use super::{get_input_query, get_wa_user_short_url, wa_query};
    use anyhow::{Error, Result};
    use mockito::{self, Matcher};

    #[test]
    fn test_input_query_content_retrieval() -> Result<(), Error> {
        let incoming = ":wa test";
        let content = get_input_query(incoming)?;
        assert_eq!(content, "test");
        Ok(())
    }

    #[test]
    fn test_input_query_content_retrieval_with_spaces() -> Result<(), Error> {
        let incoming = ":wa foo bar";
        let content = get_input_query(incoming)?;
        assert_eq!(content, "foo bar");
        Ok(())
    }

    #[test]
    fn test_input_query_content_retrieval_with_more_spaces() -> Result<(), Error> {
        let incoming = ":wa foo bar baz";
        let content = get_input_query(incoming)?;
        assert_eq!(content, "foo bar baz");
        Ok(())
    }

    // These tests must be updated if UrlShortener other than IsgdUrlShortener is used
    #[tokio::test]
    async fn test_wa_user_short_url_1() -> Result<(), Error> {
        let input = "5/10";
        assert_eq!(get_wa_user_short_url(input).await?, "https://is.gd/kgsSb7");
        Ok(())
    }

    #[tokio::test]
    async fn test_wa_user_short_url_2() -> Result<(), Error> {
        let input = "5 / 10";
        assert_eq!(get_wa_user_short_url(input).await?, "https://is.gd/b4xgwD");
        Ok(())
    }

    #[tokio::test]
    async fn test_wa_user_short_url_3() -> Result<(), Error> {
        let input = "1*2";
        assert_eq!(get_wa_user_short_url(input).await?, "https://is.gd/TLGKih");
        Ok(())
    }

    #[tokio::test]
    async fn test_wa_user_short_url_4() -> Result<(), Error> {
        let input = "1 * 2";
        assert_eq!(get_wa_user_short_url(input).await?, "https://is.gd/ZXYAUR");
        Ok(())
    }

    #[tokio::test]
    async fn test_wa_user_short_url_5() -> Result<(), Error> {
        let input = "3+4";
        assert_eq!(get_wa_user_short_url(input).await?, "https://is.gd/AZRBWy");
        Ok(())
    }

    #[tokio::test]
    async fn test_wa_user_short_url_6() -> Result<(), Error> {
        let input = "3 + 4";
        assert_eq!(get_wa_user_short_url(input).await?, "https://is.gd/lBuhgK");
        Ok(())
    }

    #[tokio::test]
    async fn test_wa_user_short_url_7() -> Result<(), Error> {
        let input = "test";
        assert_eq!(get_wa_user_short_url(input).await?, "https://is.gd/NzIyUZ");
        Ok(())
    }

    #[test]
    fn test_clean_result_text() {
        assert_eq!(
            clean_result_text("Newlines\nand  multiple\n\n whitespace   is removed."),
            "Newlines; and multiple; ; whitespace is removed.",
        )
    }

    #[tokio::test]
    async fn test_query_result_parsing() -> Result<(), Error> {
        let body = include_str!("../../tests/resources/wolfram_alpha_api_response.json");
        let _m = mockito::mock("GET", Matcher::Any)
            // Trimmed down version of a full WA response:
            .with_body(body)
            .create();
        mockito::start();

        let res = wa_query("5/10", None, Some(&mockito::server_url())).await?;
        let res_without_link = res.rsplitn(2, "-").collect::<Vec<&str>>()[1..].join(" ");
        assert_eq!(
            res_without_link.trim(),
            "Exact result: 1/2 - Decimal form: 0.5"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_query_with_spaces_result_parsing() -> Result<(), Error> {
        let body = include_str!(
            "../../tests/resources/wolfram_alpha_api_response_of_input_with_spaces.json"
        );
        let _m = mockito::mock("GET", Matcher::Any)
            // Trimmed down version of a full WA response:
            .with_body(body)
            .create();
        mockito::start();

        let res = wa_query("5 / 10", None, Some(&mockito::server_url())).await?;
        let res_without_link = res.rsplitn(2, "-").collect::<Vec<&str>>()[1..].join(" ");
        assert_eq!(
            res_without_link.trim(),
            "Exact result: 1/2 - Decimal form: 0.5"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_query_with_result_with_no_primary_pods_parsing() -> Result<(), Error> {
        let body =
            include_str!("../../tests/resources/wolfram_alpha_api_response_with_no_primaries.json");
        let _m = mockito::mock("GET", Matcher::Any)
            // Trimmed down version of a full WA response:
            .with_body(body)
            .create();
        mockito::start();

        let res = wa_query("what is a url", None, Some(&mockito::server_url())).await?;
        let res_without_link = res.rsplitn(2, "-").collect::<Vec<&str>>()[1..].join(" ");
        assert_eq!(res_without_link.trim(), "No plaintext results.");
        Ok(())
    }

    #[tokio::test]
    async fn test_query_with_result_with_wrong_json_parsing() -> Result<(), Error> {
        let body = include_str!("../../tests/resources/wolfram_alpha_api_response_wrong_json.json");
        let _m = mockito::mock("GET", Matcher::Any)
            // Trimmed down version of a full WA response:
            .with_body(body)
            .create();
        mockito::start();

        let res = wa_query("what is a url", None, Some(&mockito::server_url())).await?;
        assert_eq!(res, "No results.");
        Ok(())
    }
}
