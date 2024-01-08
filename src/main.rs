mod spotify;

use config::Config;
use crate::spotify::spotify_client::SpotifyClient;

#[tokio::main]
async fn main() {
    let properties = Config::builder().add_source(config::File::with_name("properties")).build().unwrap();
    let mut spotify_client: SpotifyClient = SpotifyClient::new(properties.get_string("spotify_client_id").unwrap());
    spotify_client.get_implicit_grant_access_token();

    println!("{}", spotify_client.get_saved_tracks().await.unwrap());
}