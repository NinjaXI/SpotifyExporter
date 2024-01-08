mod spotify;

use std::{fs, io::Write, path::Path};

use chrono::Local;
use config::Config;
use serde::{Serialize, Deserialize};
use serde_json::Value;

use crate::spotify::spotify_client::SpotifyClient;

// simple structs used to have a better json serialization for file output
#[derive(Serialize, Deserialize)]
struct TracksJson {
    tracks: Vec<Value>
}

#[derive(Serialize, Deserialize)]
struct AlbumJson {
    albums: Vec<Value>
}

#[derive(Serialize, Deserialize)]
struct AudiobookJson {
    audiobooks: Vec<Value>
}

#[derive(Serialize, Deserialize)]
struct EpisodeJson {
    episodes: Vec<Value>
}

#[derive(Serialize, Deserialize)]
struct PlaylistJson {
    playlists: Vec<Value>
}

#[derive(Serialize, Deserialize)]
struct ShowJson {
    shows: Vec<Value>
}

#[derive(Serialize, Deserialize)]
struct ArtistJson {
    artists: Vec<Value>
}

#[tokio::main]
async fn main() {
    let properties = Config::builder().add_source(config::File::with_name("properties")).build().unwrap();
    let mut spotify_client: SpotifyClient = SpotifyClient::new(properties.get_string("spotify_client_id").unwrap());
    spotify_client.get_implicit_grant_access_token();

    if !Path::new("output").exists() {
        fs::create_dir("output").unwrap();
    }

    export_saved_tracks(&spotify_client).await;
    export_saved_albums(&spotify_client).await;
    export_saved_audiobooks(&spotify_client).await;
    export_saved_episodes(&spotify_client).await;
    export_user_playlists(&spotify_client).await;
    export_saved_shows(&spotify_client).await;
    export_followed_artists(&spotify_client).await;
}

async fn export_saved_tracks(spotify_client: &SpotifyClient) {
    println!("Exporting saved tracks");

    // retrieve first 50 tracks
    let mut tracks_vector: Vec<Value> = Vec::new();
    let mut spotify_track_response: Value = spotify_client.get_saved_tracks(0, 50).await.unwrap();
    tracks_vector.append(&mut spotify_track_response["items"].as_array().unwrap().clone());

    // keep retrieving tracks until our count = total in spotify response
    while tracks_vector.len() < usize::try_from(spotify_track_response["total"].as_i64().unwrap()).unwrap() {
        spotify_track_response = spotify_client.get_saved_tracks(tracks_vector.len().try_into().unwrap(), 50).await.unwrap();
        tracks_vector.append(&mut spotify_track_response["items"].as_array().unwrap().clone());

        let percentage = (f64::from(i32::try_from(tracks_vector.len()).unwrap()) / spotify_track_response["total"].as_f64().unwrap()) * 100.0;
        print!("\rProcessing {:.0}%", percentage);
        std::io::stdout().flush().unwrap();
    }

    // save tracks as json struct to file
    fs::write(format!("output/tracks_{}.json", Local::now().format("%Y%m%d")), serde_json::to_string(&TracksJson{tracks: tracks_vector}).unwrap()).unwrap();
    println!("");
}

async fn export_saved_albums(spotify_client: &SpotifyClient) {
    println!("Exporting saved albums");

    // retrieve first 50 albums
    let mut albums_vector: Vec<Value> = Vec::new();
    let mut spotify_album_response: Value = spotify_client.get_saved_albums(0, 50).await.unwrap();
    albums_vector.append(&mut spotify_album_response["items"].as_array().unwrap().clone());

    // keep retrieving albums until our count = total in spotify response
    while albums_vector.len() < usize::try_from(spotify_album_response["total"].as_i64().unwrap()).unwrap() {
        spotify_album_response = spotify_client.get_saved_albums(albums_vector.len().try_into().unwrap(), 50).await.unwrap();
        albums_vector.append(&mut spotify_album_response["items"].as_array().unwrap().clone());

        let percentage = (f64::from(i32::try_from(albums_vector.len()).unwrap()) / spotify_album_response["total"].as_f64().unwrap()) * 100.0;
        print!("\rProcessing {:.0}%", percentage);
        std::io::stdout().flush().unwrap();
    }

    // save albums as json struct to file
    fs::write(format!("output/albums_{}.json", Local::now().format("%Y%m%d")), serde_json::to_string(&AlbumJson{albums: albums_vector}).unwrap()).unwrap();
    println!("");
}

async fn export_saved_audiobooks(spotify_client: &SpotifyClient) {
    println!("Exporting saved audiobooks");

    // retrieve first 50 audiobooks
    let mut audiobooks_vector: Vec<Value> = Vec::new();
    let mut spotify_audiobook_response: Value = spotify_client.get_saved_audiobooks(0, 50).await.unwrap();
    audiobooks_vector.append(&mut spotify_audiobook_response["items"].as_array().unwrap().clone());

    // keep retrieving audiobooks until our count = total in spotify response
    while audiobooks_vector.len() < usize::try_from(spotify_audiobook_response["total"].as_i64().unwrap()).unwrap() {
        spotify_audiobook_response = spotify_client.get_saved_albums(audiobooks_vector.len().try_into().unwrap(), 50).await.unwrap();
        audiobooks_vector.append(&mut spotify_audiobook_response["items"].as_array().unwrap().clone());

        let percentage = (f64::from(i32::try_from(audiobooks_vector.len()).unwrap()) / spotify_audiobook_response["total"].as_f64().unwrap()) * 100.0;
        print!("\rProcessing {:.0}%", percentage);
        std::io::stdout().flush().unwrap();
    }

    // save audiobooks as json struct to file
    fs::write(format!("output/audiobooks_{}.json", Local::now().format("%Y%m%d")), serde_json::to_string(&AudiobookJson{audiobooks: audiobooks_vector}).unwrap()).unwrap();
    println!("");
}

async fn export_saved_episodes(spotify_client: &SpotifyClient) {
    println!("Exporting saved episodes");

    // retrieve first 50 episodes
    let mut episodes_vector: Vec<Value> = Vec::new();
    let mut spotify_episode_response: Value = spotify_client.get_saved_episodes(0, 50).await.unwrap();
    episodes_vector.append(&mut spotify_episode_response["items"].as_array().unwrap().clone());

    // keep retrieving episodes until our count = total in spotify response
    while episodes_vector.len() < usize::try_from(spotify_episode_response["total"].as_i64().unwrap()).unwrap() {
        spotify_episode_response = spotify_client.get_saved_albums(episodes_vector.len().try_into().unwrap(), 50).await.unwrap();
        episodes_vector.append(&mut spotify_episode_response["items"].as_array().unwrap().clone());

        let percentage = (f64::from(i32::try_from(episodes_vector.len()).unwrap()) / spotify_episode_response["total"].as_f64().unwrap()) * 100.0;
        print!("\rProcessing {:.0}%", percentage);
        std::io::stdout().flush().unwrap();
    }

    // save episodes as json struct to file
    fs::write(format!("output/episodes_{}.json", Local::now().format("%Y%m%d")), serde_json::to_string(&EpisodeJson{episodes: episodes_vector}).unwrap()).unwrap();
    println!("");
}

async fn export_user_playlists(spotify_client: &SpotifyClient) {
    println!("Exporting users owned or followed playlists");

    // retrieve first 50 playlists
    let mut playlists_vector: Vec<Value> = Vec::new();
    let mut spotify_playlist_response: Value = spotify_client.get_owned_followed_playlists(0, 50).await.unwrap();
    playlists_vector.append(&mut spotify_playlist_response["items"].as_array().unwrap().clone());

    // keep retrieving playlists until our count = total in spotify response
    while playlists_vector.len() < usize::try_from(spotify_playlist_response["total"].as_i64().unwrap()).unwrap() {
        spotify_playlist_response = spotify_client.get_owned_followed_playlists(playlists_vector.len().try_into().unwrap(), 50).await.unwrap();
        playlists_vector.append(&mut spotify_playlist_response["items"].as_array().unwrap().clone());

        let percentage = (f64::from(i32::try_from(playlists_vector.len()).unwrap()) / spotify_playlist_response["total"].as_f64().unwrap()) * 100.0;
        print!("\rProcessing {:.0}%", percentage);
        std::io::stdout().flush().unwrap();
    }

    // save playlists as json struct to file
    fs::write(format!("output/playlists_{}.json", Local::now().format("%Y%m%d")), serde_json::to_string(&PlaylistJson{playlists: playlists_vector}).unwrap()).unwrap();
    println!("");
}

async fn export_saved_shows(spotify_client: &SpotifyClient) {
    println!("Exporting saved shows");

    // retrieve first 50 shows
    let mut shows_vector: Vec<Value> = Vec::new();
    let mut spotify_show_response: Value = spotify_client.get_saved_episodes(0, 50).await.unwrap();
    shows_vector.append(&mut spotify_show_response["items"].as_array().unwrap().clone());

    // keep retrieving shows until our count = total in spotify response
    while shows_vector.len() < usize::try_from(spotify_show_response["total"].as_i64().unwrap()).unwrap() {
        spotify_show_response = spotify_client.get_saved_shows(shows_vector.len().try_into().unwrap(), 50).await.unwrap();
        shows_vector.append(&mut spotify_show_response["items"].as_array().unwrap().clone());

        let percentage = (f64::from(i32::try_from(shows_vector.len()).unwrap()) / spotify_show_response["total"].as_f64().unwrap()) * 100.0;
        print!("\rProcessing {:.0}%", percentage);
        std::io::stdout().flush().unwrap();
    }

    // save shows as json struct to file
    fs::write(format!("output/shows_{}.json", Local::now().format("%Y%m%d")), serde_json::to_string(&ShowJson{shows: shows_vector}).unwrap()).unwrap();
    println!("");
}

async fn export_followed_artists(spotify_client: &SpotifyClient) {
    println!("Exporting followed artists");

    // retrieve first 50 artists
    let mut artists_vector: Vec<Value> = Vec::new();
    let mut spotify_artist_response: Value = spotify_client.get_followed_artists("", 50).await.unwrap();
    artists_vector.append(&mut spotify_artist_response["artists"]["items"].as_array().unwrap().clone());

    // keep retrieving artists until our count = total in spotify response
    let mut after: String = spotify_artist_response["artists"]["cursors"]["after"].as_str().unwrap().to_owned();
    while artists_vector.len() < usize::try_from(spotify_artist_response["artists"]["total"].as_i64().unwrap()).unwrap() {
        spotify_artist_response = spotify_client.get_followed_artists(&after, 50).await.unwrap();
        artists_vector.append(&mut spotify_artist_response["artists"]["items"].as_array().unwrap().clone());
        after = spotify_artist_response["artists"]["cursors"]["after"].as_str().unwrap_or_default().to_owned();

        let percentage = (f64::from(i32::try_from(artists_vector.len()).unwrap()) / spotify_artist_response["artists"]["total"].as_f64().unwrap()) * 100.0;
        print!("\rProcessing {:.0}%", percentage);
        std::io::stdout().flush().unwrap();
    }

    // save artists as json struct to file
    fs::write(format!("output/artists_{}.json", Local::now().format("%Y%m%d")), serde_json::to_string(&ArtistJson{artists: artists_vector}).unwrap()).unwrap();
    println!("");
}