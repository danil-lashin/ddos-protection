use serde::{Deserialize, Serialize};

pub async fn fetch_quote() -> String {
    let response = reqwest::get("https://zenquotes.io/api/random")
        .await
        .unwrap()
        .json::<Vec<ZenQuotesResponse>>()
        .await
        .unwrap();

    response[0].q.clone()
}

#[derive(Serialize, Deserialize, Debug)]
struct ZenQuotesResponse {
    q: String,
    a: String,
    h: String,
}
