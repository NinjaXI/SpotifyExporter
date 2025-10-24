# SpotifyExporter
A small script to pull your data from Spotify and save it locally as JSON files.
The reason for this is that overtime tracks that you save in Spotify could become unavailable in your region.
Currently when this happens you can still see them in your liked list, but they are greyed out.
Notably you can't see these anywhere else in the app(eg when searching for the exact etc.).
This saves the data locally so that should Spotify deprecate support for liked tracks that are no longer available in your region, you still have record of them if you want to find them elsewhere.

# Output
Currently this script saves all user specific data from the API as JSON files.
The JSON structures are left mostly intact(except for playlist export) as it is intended to be a simple dump processed seperately.
The data exported : 
   - Liked Songs
   - Liked Albums
   - Liked Audiobooks
   - Liked Podcast Episodes
   - Followed and Created Playlists
   - Liked Shows
   - Followed Artists

Output can be found in the output folder in seperate JSON files with dates in the filename.

# Usage
1. Download the relevant release
2. Create an app on the Spotify Web API as instructed here : https://developer.spotify.com/documentation/web-api
   - Take note of the client ID and secret generated
   - Set your redirect URI to http://localhost:8000/callback
3. Rename properties.default.toml to properties.toml
4. After renaming update `oauth_flow_type` to preferred OAuth2.0 flow type(Implicit Grant by default).
5. Update `spotify_client_id` to the client ID from step 1.
6. Update `spotify_client_secret` to the client secret from step 1. 
7. To run simply execute the binary depending what platform its on "spotify-exporter.exe" for Windows for example.
9. Additional options : 
   - -t, --token generates the refresh token without performing export, useful to generate the token and then use it elsewhere on a headless server
   - -z, --zip indicates whether to zip the exported files automatically after export

# Project Setup
This is a relatively simple script so the setup should be quick and easy.
1. Install rust and setup rust
2. Do steps 2-6 from the Usage section as you will need those in place to test any changes
3. Build using cargo build
4. Run using cargo run
