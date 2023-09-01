use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ServerInfo {
    pub uuid: String,
    pub name: String,
}

pub async fn get_servers() -> Result<Vec<ServerInfo>, reqwest::Error> {
    let client = reqwest::Client::new();
    let result = client
        .get(std::env::var("SERVER_API").unwrap())
        .send()
        .await;

    match result {
        Ok(result) => result.json().await,
        Err(e) => Err(e),
    }
}
