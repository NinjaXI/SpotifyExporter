use rand::{distributions::Alphanumeric, Rng};
use reqwest::{Client, Response, Error};
use serde_json::Value;
use base64::{prelude::*};

use std::{collections::HashMap, fs::{self, File}, io::{Read, Write}, net::{TcpListener, TcpStream}, time::{SystemTime, UNIX_EPOCH}};

pub struct SpotifyClient {
    flow_type: String,
    spotify_client_id: String,
    spotify_client_secret: String,
    access_token: String,
    refresh_token: String,
    token_type: String,
    expires_in: u64,
    token_refreshed: u64,
    client: Client
}

impl SpotifyClient {

    pub fn new(flow_type: String, spotify_client_id: String, spotify_client_secret: String) -> Self {
        let access_token: String = "".to_string();
        let refresh_token: String = "".to_string();
        let token_type: String = "".to_string();
        let expires_in: u64 = 0;
        let token_refreshed: u64 = 0;
        let client = Client::new();

        Self {
            flow_type,
            spotify_client_id,
            spotify_client_secret,
            access_token,
            refresh_token,
            token_type,
            token_refreshed,
            expires_in,
            client
        }
    }

    /// Get access token for Spotify API
    /// 
    /// This will open a browser window from Spotify asking the user to grant the privelages required to this script.
    /// Once granted, Spotify will do a callback request which the script will catch and serve a callback html for.
    /// This html file will, using javascript, extract the query parameters and do a request back to this script so that we can extract the access token here in the backend.
    pub async fn get_access_token(&mut self) -> Result<bool, Error> {
        let mut has_token: bool = false;
        if self.flow_type.eq("code") {
            if fs::exists("token.txt").unwrap() {
                self.refresh_token = fs::read_to_string("token.txt").unwrap();

                has_token = self.refresh_access_token_validity().await?;
            }
        }

        if !has_token {
            // start TCP Listener that will be used to receive callback requests as part of OAuth flow
            let listener: TcpListener = TcpListener::bind("127.0.0.1:8000").expect("Failed to bind to address");

            // generate random 16 length string to validate in implicit grant
            let state: String = rand::thread_rng().sample_iter(&Alphanumeric).take(16).map(char::from).collect();
            let scope: &str = "user-library-read user-read-playback-position playlist-read-private user-follow-read";
            let authorization_url: &str = &format!("https://accounts.spotify.com/authorize?response_type={}&client_id={}&scope={}&redirect_uri=http://localhost:8000/callback&state={}", self.flow_type, self.spotify_client_id, scope, state);
            open::that(authorization_url).unwrap();

            println!("auth url {} ", authorization_url);
            std::io::stdout().flush().unwrap();
            let mut running: bool = true;
            while running {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        let mut buffer = [0; 1024];
                        stream.read(&mut buffer).expect("Failed to read request");

                        let request: &str = std::str::from_utf8(&buffer).unwrap();

                        let first_line: &str = request.lines().next().unwrap();
                        let url: &str = first_line.split_whitespace().nth(1).unwrap();

                        // we only expect 2 calls here, either the callback from spotify, or a finalize call from our own html
                        if url.contains("finalizeAuthentication") {
                            // if it is the finalize call we extract the relevant details from the URL and finalize the oauth flow
                            if self.flow_type.eq("token") {
                                self.finalize_implicit_grant(url, &state);
                            } else {
                                self.finalize_authorization_code(url, &state).await?; // TODO authorization code with PKCE, maybe?
                            }

                            running = false;
                        } else {
                            // if its not the finalize call we assume its the callback from spotify and serve our callback html
                            self.serve_callback(&mut stream);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }
        }

        Ok(true)
    }

    /// Retrieve the saved tracks for the user
    ///
    /// # Arguments
    ///
    /// * `offset` - An int that specifies the offset in the list of saved tracks
    /// * `limit` - An int specifying total number of tracks to return, 50 is max
    pub async fn get_saved_tracks(&mut self, offset: i32, limit: i32) -> Result<Value, Error> {
        if !self.refresh_access_token_validity().await.unwrap() {
            panic!("No valid token")
        }

        let url: String = format!("https://api.spotify.com/v1/me/tracks?offset={}&limit={}", offset, limit);
        let get_response: Response = self.client.get(url).header("Authorization", format!("{} {}", self.token_type, self.access_token)).send().await?;
        let get_response_json: Value = serde_json::from_str(&get_response.text().await?).expect("JSON was not well-formatted");

        Ok(get_response_json)
    }

    /// Retrieve the saved albums for the user
    ///
    /// # Arguments
    ///
    /// * `offset` - An int that specifies the offset in the list of saved albums
    /// * `limit` - An int specifying total number of albums to return, 50 is max
    pub async fn get_saved_albums(&mut self, offset: i32, limit: i32) -> Result<Value, Error> {
        if !self.refresh_access_token_validity().await.unwrap() {
            panic!("No valid token")
        }

        let url: String = format!("https://api.spotify.com/v1/me/albums?offset={}&limit={}", offset, limit);
        let get_response: Response = self.client.get(url).header("Authorization", format!("{} {}", self.token_type, self.access_token)).send().await?;
        let get_response_json: Value = serde_json::from_str(&get_response.text().await?).expect("JSON was not well-formatted");

        Ok(get_response_json)
    }

    /// Retrieve the saved audiobooks for the user
    ///
    /// # Arguments
    ///
    /// * `offset` - An int that specifies the offset in the list of saved audiobooks
    /// * `limit` - An int specifying total number of audiobooks to return, 50 is max
    pub async fn get_saved_audiobooks(&mut self, offset: i32, limit: i32) -> Result<Value, Error> {
        if !self.refresh_access_token_validity().await.unwrap() {
            panic!("No valid token")
        }

        let url: String = format!("https://api.spotify.com/v1/me/audiobooks?offset={}&limit={}", offset, limit);
        let get_response: Response = self.client.get(url).header("Authorization", format!("{} {}", self.token_type, self.access_token)).send().await?;
        let get_response_json: Value = serde_json::from_str(&get_response.text().await?).expect("JSON was not well-formatted");

        Ok(get_response_json)
    }

    /// Retrieve the saved episodes for the user
    ///
    /// # Arguments
    ///
    /// * `offset` - An int that specifies the offset in the list of saved episodes
    /// * `limit` - An int specifying total number of episodes to return, 50 is max
    pub async fn get_saved_episodes(&mut self, offset: i32, limit: i32) -> Result<Value, Error> {
        if !self.refresh_access_token_validity().await.unwrap() {
            panic!("No valid token")
        }

        let url: String = format!("https://api.spotify.com/v1/me/episodes?offset={}&limit={}", offset, limit);
        let get_response: Response = self.client.get(url).header("Authorization", format!("{} {}", self.token_type, self.access_token)).send().await?;
        let get_response_json: Value = serde_json::from_str(&get_response.text().await?).expect("JSON was not well-formatted");

        Ok(get_response_json)
    }

    /// Retrieve the owned or followed playlists for the user
    ///
    /// # Arguments
    ///
    /// * `offset` - An int that specifies the offset in the list of playlists
    /// * `limit` - An int specifying total number of playlists to return, 50 is max
    pub async fn get_owned_followed_playlists(&mut self, offset: i32, limit: i32) -> Result<Value, Error> {
        if !self.refresh_access_token_validity().await.unwrap() {
            panic!("No valid token")
        }

        let url: String = format!("https://api.spotify.com/v1/me/playlists?offset={}&limit={}", offset, limit);
        let get_response: Response = self.client.get(url).header("Authorization", format!("{} {}", self.token_type, self.access_token)).send().await?;
        let get_response_json: Value = serde_json::from_str(&get_response.text().await?).expect("JSON was not well-formatted");

        Ok(get_response_json)
    }

    /// Retrieve the tracks of the playlist for the given playlist id
    ///
    /// # Arguments
    ///
    /// * `playlist_id` - The id of the playlist to retrieve tracks for
    /// * `offset` - An int that specifies the offset in the list of tracks
    /// * `limit` - An int specifying total number of tracks to return, 50 is max
    pub async fn get_playlist_tracks(&mut self, playlist_id: &str, offset: i32, limit: i32) -> Result<Value, Error> {
        if !self.refresh_access_token_validity().await.unwrap() {
            panic!("No valid token")
        }

        let fields: &str = "items(added_by.id,added_at,track(id,name,album(album_type,name,release_date,artists(id,name)),artists(id,name)))"; // the fields specifier for track.album.artists has no affect
        let url: String = format!("https://api.spotify.com/v1/playlists/{}/tracks?fields={}&offset={}&limit={}", playlist_id, fields, offset, limit);
        let get_response: Response = self.client.get(url).header("Authorization", format!("{} {}", self.token_type, self.access_token)).send().await?;
        let get_response_json: Value = serde_json::from_str(&get_response.text().await?).expect("JSON was not well-formatted");

        Ok(get_response_json)
    }

    /// Retrieve the saved shows for the user
    ///
    /// # Arguments
    ///
    /// * `offset` - An int that specifies the offset in the list of saved shows
    /// * `limit` - An int specifying total number of shows to return, 50 is max
    pub async fn get_saved_shows(&mut self, offset: i32, limit: i32) -> Result<Value, Error> {
        if !self.refresh_access_token_validity().await.unwrap() {
            panic!("No valid token")
        }

        let url: String = format!("https://api.spotify.com/v1/me/shows?offset={}&limit={}", offset, limit);
        let get_response: Response = self.client.get(url).header("Authorization", format!("{} {}", self.token_type, self.access_token)).send().await?;
        let get_response_json: Value = serde_json::from_str(&get_response.text().await?).expect("JSON was not well-formatted");

        Ok(get_response_json)
    }

    /// Retrieve the followed artists for the user
    ///
    /// # Arguments
    ///
    /// * `offset` - An int that specifies the offset in the list of followed artists
    /// * `limit` - An int specifying total number of artists to return, 50 is max
    pub async fn get_followed_artists(&mut self, after: &str, limit: i32) -> Result<Value, Error> {
        if !self.refresh_access_token_validity().await.unwrap() {
            panic!("No valid token")
        }

        let url: String;
        if after == "" {
            url = format!("https://api.spotify.com/v1/me/following?type=artist&limit={}", limit);
        } else {
            url = format!("https://api.spotify.com/v1/me/following?type=artist&after={}&limit={}", after, limit);
        }

        let get_response: Response = self.client.get(url).header("Authorization", format!("{} {}", self.token_type, self.access_token)).send().await?;
        let get_response_json: Value = serde_json::from_str(&get_response.text().await?).expect("JSON was not well-formatted");

        Ok(get_response_json)
    }

    /// Serves the html file in src/html/callback.html as response on the TcpStream
    ///
    /// # Arguments
    ///
    /// * `stream` - The TCP stream to serve the response on
    fn serve_callback(&mut self, stream: &mut TcpStream) {
        let content: String = std::fs::read_to_string("src/html/callback.html").unwrap_or_else(|_| {
            "Failed to read the HTML file".to_string()
        });

        let response: String = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n{}",
            content
        );

        stream.write(response.as_bytes()).expect("Failed to write response");
        stream.flush().expect("Failed to flush stream");
    }

    /// Extracts the access_token and other properties for the Spotify API from the url
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to extract the query paramters from
    /// * `state` - State string provided to Spotify in initial request that must match
    fn finalize_implicit_grant(&mut self, url: &str, state: &str) {
        let query_params = url.split("?").nth(1).unwrap().split("&");

        for param in query_params {
            let mut param_arr = param.split("=");
            match param_arr.next().unwrap() {
                "access_token" => {
                    self.access_token = param_arr.next().unwrap().to_owned();
                }, "token_type" => {
                    self.token_type = param_arr.next().unwrap().to_owned();
                }, "expires_in" => {
                    self.expires_in = param_arr.next().unwrap().parse().unwrap();
                }, "state" => {
                    if state != param_arr.next().unwrap() {
                        panic!("State does not match")
                    }
                }, _ => {
                    // dont care
                }
            }
        }
    }

    /// Extracts the code for the Spotify API from the url
    /// Then continues the OAuth2.0 Authorization Code flow by using the code to request an access token
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to extract the query paramters from
    /// * `state` - State string provided to Spotify in initial request that must match
    async fn finalize_authorization_code(&mut self, url: &str, state: &str) -> Result<bool, Error> {
        let query_params = url.split("?").nth(1).unwrap().split("&");

        for param in query_params {
            let mut param_arr = param.split("=");
            match param_arr.next().unwrap() {
                "code" => {
                    let authorization_code: String = param_arr.next().unwrap().to_owned();
                    let mut form_params = HashMap::new();
                    form_params.insert("grant_type", "authorization_code");
                    form_params.insert("code", &authorization_code);
                    form_params.insert("redirect_uri", "http://localhost:8000/callback");

                    let access_token_url: &str = "https://accounts.spotify.com/api/token";
                    let auth_header: String = format!("Basic {}", BASE64_STANDARD.encode(format!("{}:{}", self.spotify_client_id, self.spotify_client_secret)));
                    let access_token_response: Response = self.client.post(access_token_url)
                                                                .header("Authorization", auth_header)
                                                                .header("Content-Type", "application/x-www-form-urlencoded")
                                                                .form(&form_params)
                                                                .send().await?;
                    let access_token_response_json: Value = serde_json::from_str(&access_token_response.text().await?).expect("JSON was not well-formatted");
                    self.access_token = access_token_response_json["access_token"].as_str().unwrap().to_owned();
                    self.refresh_token = access_token_response_json["refresh_token"].as_str().unwrap().to_owned();
                    self.token_type = access_token_response_json["token_type"].as_str().unwrap().to_owned();
                    self.expires_in = access_token_response_json["expires_in"].as_u64().unwrap().to_owned();
                    self.token_refreshed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

                    let mut token_file: File = File::create("token.txt").unwrap();
                    token_file.write_all(self.refresh_token.as_bytes()).unwrap();
                }, "state" => {
                    if state != param_arr.next().unwrap() {
                        panic!("State does not match")
                    }
                }, _ => {
                    // dont care
                }
            }
        }

        Ok(true)
    }

    /// Uses the stored refresh token to refresh the access token if it has expired or has not been retrieved yet.
    /// 
    /// # Returns
    /// True if a valid token has been retrieved
    async fn refresh_access_token_validity(&mut self) -> Result<bool, Error> {
        // only refresh token for authorization code flow
        if self.flow_type.eq("code") {
            return Ok(true)
        }

        let now_secs: u64 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        if self.token_refreshed == 0 || (self.token_refreshed + self.expires_in) > (now_secs - 300) {
            let mut form_params = HashMap::new();
            form_params.insert("grant_type", "refresh_token");
            form_params.insert("refresh_token", &self.refresh_token);

            let refresh_token_url: &str = "https://accounts.spotify.com/api/token";
            let auth_header: String = format!("Basic {}", BASE64_STANDARD.encode(format!("{}:{}", self.spotify_client_id, self.spotify_client_secret)));
            let access_token_response: Response = self.client.post(refresh_token_url)
                                                        .header("Authorization", auth_header)
                                                        .header("Content-Type", "application/x-www-form-urlencoded")
                                                        .form(&form_params)
                                                        .send().await?;
            let access_token_response_json: Value = serde_json::from_str(&access_token_response.text().await?).expect("JSON was not well-formatted");
            self.access_token = access_token_response_json["access_token"].as_str().unwrap().to_owned();
            self.token_type = access_token_response_json["token_type"].as_str().unwrap().to_owned();
            self.expires_in = access_token_response_json["expires_in"].as_u64().unwrap().to_owned();

            if !access_token_response_json["refresh_token"].is_null() {
                self.refresh_token = access_token_response_json["refresh_token"].as_str().unwrap().to_owned();

                let mut token_file: File = File::create("token.txt").unwrap();
                token_file.write_all(self.refresh_token.as_bytes()).unwrap();
            }
        }

        Ok(true)
    }
}