use reqwest::{Client, Response, Error};
use serde_json::Value;

use std::{net::{TcpStream, TcpListener}, io::{Read, Write}};

pub struct SpotifyClient {
    spotify_client_id: String,
    access_token: String,
    token_type: String,
    expires_in: i32,
    client: Client
}

impl SpotifyClient {

    pub fn new(spotify_client_id: String) -> Self {
        let access_token: String = "".to_string();
        let token_type: String = "".to_string();
        let expires_in: i32 = 0;
        let client = Client::new();

        Self {
            spotify_client_id,
            access_token,
            token_type,
            expires_in,
            client
        }
    }

    /// Get implicit grant access token for Spotify API
    /// 
    /// This will open a browser window from Spotify asking the user to grant the privelages required to this script.
    /// Once granted, Spotify will do a callback request which the script will catch and serve a callback html for.
    /// This html file will, using javascript, extract the query parameters and do a request back to this script so that we can extract the access token here in the backend.
    pub fn get_implicit_grant_access_token(&mut self) {
        // start TCP Listener that will be used to receive callback requests as part of OAuth flow
        let listener: TcpListener = TcpListener::bind("127.0.0.1:8000").expect("Failed to bind to address");

        // TODO genereate random string for state
        let state: &str = "1234567890123456";
        let scope: &str = "user-read-private user-read-email user-library-read";
        let authorization_url: &str = &format!("https://accounts.spotify.com/authorize?response_type=token&client_id={}&scope={}&redirect_uri=http://localhost:8000/callback&state={}", self.spotify_client_id, scope, state);
        open::that(authorization_url).unwrap();

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
                        // if it is the finalize call we extract the access_token from the url
                        self.finalize_implicit_grant(url, state);

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

    /// Retrieve the saved tracks for the user
    ///
    /// # Arguments
    ///
    /// * `offset` - An int that specifies the offset in the list of saved tracks
    /// * `limit` - An int specifying total number of tracks to return, 50 is max
    pub async fn get_saved_tracks(&self, offset: i32, limit: i32) -> Result<Value, Error> {
        let url: String = format!("https://api.spotify.com/v1/me/tracks?offset={}&limit={}", offset, limit);
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

    // no longer used, but left here for reference
    // pub async fn get_access_token(&mut self) -> Result<(), Error> {
    //     let token_url: &str = "https://accounts.spotify.com/api/token";

    //     let request_json: String = format!("grant_type=client_credentials&client_id={}&client_secret={}", self.spotify_client_id, self.spotify_client_secret);
    //     let response: Response = self.client
    //         .post(token_url)
    //         .header("Content-Type", "application/x-www-form-urlencoded")
    //         .body(request_json)
    //         .send()
    //         .await?;


    //     let response_json: Value = serde_json::from_str(&response.text().await.unwrap()).expect("JSON was not well-formatted");
    //     self.access_token = response_json["access_token"].as_str().unwrap_or_default().to_string();

    //     Ok(())
    // }
}