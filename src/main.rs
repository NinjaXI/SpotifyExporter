use reqwest::{Client, Response, Error};
use serde_json::{Value};
use config::{Config};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let properties = Config::builder().add_source(config::File::with_name("properties")).build().unwrap();
    let spotify_client_id: &str = &properties.get_string("spotify_client_id").unwrap();
    let spotify_client_secret: &str = &properties.get_string("spotify_client_secret").unwrap();
    let token_url: &str = "https://accounts.spotify.com/api/token";

    let client: Client = reqwest::Client::new();
    let request_json: String = format!("grant_type=client_credentials&client_id={}&client_secret={}", spotify_client_id, spotify_client_secret);
    let response: Response = client
        .post(token_url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(request_json)
        .send()
        .await?;

    let response_json: Value = serde_json::from_str(&response.text().await?).expect("JSON was not well-formatted");
    let access_token: &str = response_json["access_token"].as_str().unwrap_or_default();

    let get_response: Response = client.get("https://api.spotify.com/v1/artists/4Z8W4fKeB5YxbusRsdQVPb").header("Authorization", format!("Bearer {}", access_token)).send().await?;
    let get_response_json: Value = serde_json::from_str(&get_response.text().await?).expect("JSON was not well-formatted");
    println!("{}", get_response_json);
    Ok(())
}
