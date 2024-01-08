mod spotify;

use reqwest::{Error};
use config::{Config};

use crate::spotify::spotify_client::SpotifyClient;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let properties = Config::builder().add_source(config::File::with_name("properties")).build().unwrap();
    let mut spotify_client: SpotifyClient = SpotifyClient::new(properties.get_string("spotify_client_id").unwrap(), properties.get_string("spotify_client_secret").unwrap());
    spotify_client.get_access_token().await?;

    match spotify_client.get_artist("4Z8W4fKeB5YxbusRsdQVPb").await {
        Ok(artist) => println!("{}", artist),
        Err(error) => println!("Error occurred while retrieving artist {}", error.to_string())
    };

    Ok(())
}