use reqwest::header::{HeaderMap, AUTHORIZATION};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ServerInfo {
    pub uuid: String,
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct ServerObject {
    attributes: ServerInfo,
}

#[derive(Deserialize, Debug)]
pub struct Resp {
    data: Vec<ServerObject>,
}

pub async fn get_servers() -> Result<Vec<ServerInfo>, reqwest::Error> {
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        format!("Bearer {}", std::env::var("SERVER_KEY").unwrap())
            .parse()
            .unwrap(),
    );

    let result = client
        .get(format!(
            "{}/api/application/servers",
            std::env::var("SERVER_API").unwrap()
        ))
        .headers(headers)
        .send()
        .await?;
    let response = result.json::<Resp>().await.unwrap();
    Ok(response.data.into_iter().map(|d| d.attributes).collect())
}
