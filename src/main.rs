mod spotify;

use rocket::{get, Rocket, Build, http::Status};

use config::Config;
use crate::spotify::spotify_client::SpotifyClient;

#[macro_use]
extern crate rocket;

#[launch]
fn launch() -> Rocket<Build> {
    rocket::build().mount("/", routes![get_artist])
}

// artistId for testing : 4Z8W4fKeB5YxbusRsdQVPb
#[get("/artist/<artist_id>")]
async fn get_artist(artist_id: &str) -> Result<(), Status> {
    let properties = Config::builder().add_source(config::File::with_name("properties")).build().unwrap();
    let mut spotify_client: SpotifyClient = SpotifyClient::new(properties.get_string("spotify_client_id").unwrap(), properties.get_string("spotify_client_secret").unwrap());
    spotify_client.get_access_token().await.expect("Could not retrieve access token");

    match spotify_client.get_artist(artist_id).await {
        Ok(artist) => println!("{}", artist),
        Err(error) => println!("Error occurred while retrieving artist {}", error.to_string())
    };

    Ok(())
}