mod spotify;

use std::{fs, io::Write};

use chrono::Local;
use config::Config;
use serde::{Serialize, Deserialize};
use serde_json::Value;

use crate::spotify::spotify_client::SpotifyClient;

// simple struct used to have a better json serialization for file output
#[derive(Serialize, Deserialize)]
struct TracksJson {
    tracks: Vec<Value>
}

#[tokio::main]
async fn main() {
    let properties = Config::builder().add_source(config::File::with_name("properties")).build().unwrap();
    let mut spotify_client: SpotifyClient = SpotifyClient::new(properties.get_string("spotify_client_id").unwrap());
    spotify_client.get_implicit_grant_access_token();

    println!("Exporting saved tracks");

    // retrieve first 50 tracks
    let mut tracks_vector: Vec<Value> = Vec::new();
    let mut spotify_track_response: Value = spotify_client.get_saved_tracks(0, 50).await.unwrap();
    tracks_vector.append(&mut spotify_track_response["items"].as_array().unwrap().clone());

    // keep retrieving tracks untill the our count = total in spotify response
    while tracks_vector.len() < usize::try_from(spotify_track_response["total"].as_i64().unwrap()).unwrap() {
        spotify_track_response = spotify_client.get_saved_tracks(tracks_vector.len().try_into().unwrap(), 50).await.unwrap();
        tracks_vector.append(&mut spotify_track_response["items"].as_array().unwrap().clone());

        let percentage = (f64::from(i32::try_from(tracks_vector.len()).unwrap()) / spotify_track_response["total"].as_f64().unwrap()) * 100.0;
        print!("\rProcessing {:.0}%", percentage);
        std::io::stdout().flush().unwrap();
    }

    // save tracks as json struct to file
    fs::write(format!("tracks_{}.json", Local::now().format("%Y%m%d")), serde_json::to_string(&TracksJson{tracks: tracks_vector}).unwrap()).unwrap();
}