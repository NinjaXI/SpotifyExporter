use reqwest::{Client, Response, Error};
use serde_json::Value;

pub struct SpotifyClient {
    spotify_client_id: String,
    spotify_client_secret: String,
    access_token: String,
    client: Client
}

impl SpotifyClient {

    pub fn new(spotify_client_id: String, spotify_client_secret: String) -> Self {
        let access_token: String = "".to_string();
        let client = Client::new();
        Self {
            spotify_client_id,
            spotify_client_secret,
            access_token,
            client
        }
    }

    pub async fn get_access_token(&mut self) -> Result<(), Error> {
        let token_url: &str = "https://accounts.spotify.com/api/token";

        let request_json: String = format!("grant_type=client_credentials&client_id={}&client_secret={}", self.spotify_client_id, self.spotify_client_secret);
        let response: Response = self.client
            .post(token_url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(request_json)
            .send()
            .await?;


        let response_json: Value = serde_json::from_str(&response.text().await.unwrap()).expect("JSON was not well-formatted");
        self.access_token = response_json["access_token"].as_str().unwrap_or_default().to_string();

        Ok(())
    }

    pub async fn get_artist(&self, artist_id: &str) -> Result<String, Error> {
        let get_response: Response = self.client.get("https://api.spotify.com/v1/artists/".to_string() + artist_id).header("Authorization", format!("Bearer {}", self.access_token)).send().await?;
        let get_response_json: Value = serde_json::from_str(&get_response.text().await?).expect("JSON was not well-formatted");

        Ok(get_response_json.to_string())
    }
}