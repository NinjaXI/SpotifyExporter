mod spotify;

use std::{fs::{self, File}, io::Write, path::{Path, PathBuf}, str::FromStr};

use chrono::Local;
use clap::Parser;
use config::Config;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use zip::write::SimpleFileOptions;

use crate::spotify::spotify_client::SpotifyClient;

#[derive(Parser)]
#[command(version, 
    about="Exports all your saved data from Spotify", 
    long_about = None)]
struct Args {
    /// only retrieve refresh token to be used for authorization code flow, no exporting performed
    #[arg(short, long)]
    token: bool,
    /// zip exported files
    #[arg(short, long)]
    zip: bool
}

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
    let args = Args::parse();

    let properties = Config::builder().add_source(config::File::with_name("properties")).build().unwrap();
    let mut spotify_client: SpotifyClient = SpotifyClient::new(properties.get_string("oauth_flow_type").unwrap(), properties.get_string("spotify_client_id").unwrap(), properties.get_string("spotify_client_secret").unwrap());
    spotify_client.get_access_token().await.unwrap();

    if args.token {
        println!("Token retrieved and saved, please see token.txt");
        std::io::stdout().flush().unwrap();
        return;
    }

    if !Path::new("output").exists() {
        fs::create_dir("output").unwrap();
    }

    export_saved_tracks(&mut spotify_client).await;
    export_saved_albums(&mut spotify_client).await;
    export_saved_audiobooks(&mut spotify_client).await;
    export_saved_episodes(&mut spotify_client).await;
    export_user_playlists(&mut spotify_client).await;
    export_saved_shows(&mut spotify_client).await;
    export_followed_artists(&mut spotify_client).await;

    if args.zip {
        zip_exported_json().unwrap();
    }
}

async fn export_saved_tracks(spotify_client: &mut SpotifyClient) {
    println!("Exporting saved tracks");
    print!("\rProcessing 0%");
    std::io::stdout().flush().unwrap();

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

    print!("\rProcessing 100%\n");
    std::io::stdout().flush().unwrap();
}

async fn export_saved_albums(spotify_client: &mut SpotifyClient) {
    println!("Exporting saved albums");
    print!("\rProcessing 0%");
    std::io::stdout().flush().unwrap();

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

    print!("\rProcessing 100%\n");
    std::io::stdout().flush().unwrap();
}

async fn export_saved_audiobooks(spotify_client: &mut SpotifyClient) {
    println!("Exporting saved audiobooks");
    print!("\rProcessing 0%");
    std::io::stdout().flush().unwrap();

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

    print!("\rProcessing 100%\n");
    std::io::stdout().flush().unwrap();
}

async fn export_saved_episodes(spotify_client: &mut SpotifyClient) {
    println!("Exporting saved episodes");
    print!("\rProcessing 0%");
    std::io::stdout().flush().unwrap();

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

    print!("\rProcessing 100%\n");
    std::io::stdout().flush().unwrap();
}

async fn export_user_playlists(spotify_client: &mut SpotifyClient) {
    println!("Exporting users owned or followed playlists");
    print!("\rProcessing 0%");
    std::io::stdout().flush().unwrap();

    // retrieve first 50 playlists
    let mut playlists_vector: Vec<Value> = Vec::new();
    let mut spotify_playlist_response: Value = spotify_client.get_owned_followed_playlists(0, 50).await.unwrap();
    for mut playlist in spotify_playlist_response["items"].as_array().unwrap().to_owned() {
        let spotify_track_response: Value = spotify_client.get_playlist_tracks(playlist["id"].as_str().unwrap(), 0, 50).await.unwrap();
        playlist["tracks"] = spotify_track_response["items"].clone();

        playlists_vector.push(playlist);
    }

    // keep retrieving playlists until our count = total in spotify response
    while playlists_vector.len() < usize::try_from(spotify_playlist_response["total"].as_i64().unwrap()).unwrap() {
        spotify_playlist_response = spotify_client.get_owned_followed_playlists(playlists_vector.len().try_into().unwrap(), 50).await.unwrap();
        for mut playlist in spotify_playlist_response["items"].as_array().unwrap().to_owned() {
            let spotify_track_response: Value = spotify_client.get_playlist_tracks(playlist["id"].as_str().unwrap(), 0, 50).await.unwrap();
            playlist["tracks"] = spotify_track_response["items"].clone();
    
            playlists_vector.push(playlist);
        }

        let percentage = (f64::from(i32::try_from(playlists_vector.len()).unwrap()) / spotify_playlist_response["total"].as_f64().unwrap()) * 100.0;
        print!("\rProcessing {:.0}%", percentage);
        std::io::stdout().flush().unwrap();
    }

    // save playlists as json struct to file
    fs::write(format!("output/playlists_{}.json", Local::now().format("%Y%m%d")), serde_json::to_string(&PlaylistJson{playlists: playlists_vector}).unwrap()).unwrap();

    print!("\rProcessing 100%\n");
    std::io::stdout().flush().unwrap();
}

async fn export_saved_shows(spotify_client: &mut SpotifyClient) {
    println!("Exporting saved shows");
    print!("\rProcessing 0%");
    std::io::stdout().flush().unwrap();

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

    print!("\rProcessing 100%\n");
    std::io::stdout().flush().unwrap();
}

async fn export_followed_artists(spotify_client: &mut SpotifyClient) {
    println!("Exporting followed artists");
    print!("\rProcessing 0%");
    std::io::stdout().flush().unwrap();

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

    print!("\rProcessing 100%\n");
    std::io::stdout().flush().unwrap();
}

fn zip_exported_json() -> zip::result::ZipResult<()> {
    println!("Zipping exported files");
    std::io::stdout().flush().unwrap();

    let zip_file: File = File::create(format!("output/{}_exported.zip", Local::now().format("%Y%m%d"))).unwrap();
    let mut zip_writer = zip::ZipWriter::new(zip_file);

    let output_path_dir: PathBuf = PathBuf::from_str("output").unwrap();
    for entry in output_path_dir.read_dir().unwrap() {
        match entry {
            Ok(e) => {
                if e.path().is_file() && e.path().to_string_lossy().ends_with(&format!("{}.json", Local::now().format("%Y%m%d"))) {
                    zip_writer.start_file(e.path().file_name().unwrap().to_string_lossy(), SimpleFileOptions::default()).unwrap();

                    let mut to_zip_file: File = File::open(e.path()).unwrap();
                    std::io::copy(&mut to_zip_file, &mut zip_writer).unwrap();
                    std::fs::remove_file(e.path()).unwrap();
                }
            }
            Err(_) => todo!(),
        }
    }

    zip_writer.finish().unwrap();
    Ok(())
}