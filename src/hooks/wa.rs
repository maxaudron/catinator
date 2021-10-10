use anyhow::{bail, Context, Error, Result};
use reqwest::{get, Url};
use serde::{Deserialize, Serialize};
use serde_json::Result as SerdeJsonResult;
use tracing::trace;

#[derive(Serialize, Deserialize, Debug)]
struct WaResult {
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
    subpods: Vec<SubPod>,
}

#[derive(Serialize, Deserialize, Debug)]
struct SubPod {
    title: String,
    plaintext: String,
}

fn parse_json(str_data: &str) -> SerdeJsonResult<WaResult> {
    let w: WaResult = serde_json::from_str(str_data)?;
    Ok(w)
}

#[tracing::instrument]
async fn wa_query(query_str: &str) -> Result<String, Error> {
    let app_id = "XXX"; // TODO: Get from env
    let api_url = format!(
        "http://api.wolframalpha.com/v2/query?input={}&appid={}&output=json",
        query_str, app_id
    );

    let body = get(Url::parse(&api_url).context("Failed to parse url")?)
        .await
        .context("Failed to make request")?
        .text()
        .await
        .context("failed to get request response text")?;

    let full_wa_res = parse_json(&body)?;
    trace!("got full_wa_res: {:?}", full_wa_res);

    let pod_plaintexts = full_wa_res.queryresult.pods
        .iter()
        .filter(|it| it.id.to_lowercase() != "input")
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
        .join(" - ");

    Ok(pod_plaintexts)
}

#[cfg(test)]
mod tests {

    use super::wa_query;
    use anyhow::{Error, Result};

    #[tokio::test]
    // async fn test_query_result_json_parsing() -> Result<(), Error> {
    async fn test_query_result_json_parsing() {
        let res = wa_query("weather graz").await.unwrap();
        assert_eq!(res, "asdf");
    }
}
