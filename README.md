# SpotifyExporter
A small script to pull your data from Spotify and save it locally as JSON files.
The reason for this is that overtime tracks that you save in Spotify could become unavailable in your region.
Currently when this happens you can still see them in your liked list, but they are greyed out.
Notably you can't see these anywhere else in the app(eg when searching for the exact track etc.).
This saves the data locally so that should Spotify deprecate support for liked tracks that are no longer available in your region, you still have record of them if you want to find them elsewhere.

This was ported using Claude from Rust(original repo at https://github.com/NinjaXI/SpotifyExporterRust) to Python. 
The original script was written as a learning exercise for myself and I don't keep the Rust toolchains installed/updated to fix bugs etc. Ported to Python I can maintain this easier. 

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
1. Install Python 3.11 or newer
2. Install the dependencies : `pip install -r requirements.txt`
3. Create an app on the Spotify Web API as instructed here : https://developer.spotify.com/documentation/web-api
   - Take note of the client ID and secret generated
   - Set your redirect URI to http://localhost:8000/callback
4. Rename properties.default.toml to properties.toml
5. After renaming update `oauth_flow_type` to preferred OAuth2.0 flow type(Implicit Grant by default).
6. Update `spotify_client_id` to the client ID from step 3.
7. Update `spotify_client_secret` to the client secret from step 3.
8. To run, execute `python src/main.py` from the project folder(the script reads properties.toml, token.txt and src/html/callback.html relative to the current folder).
9. Additional options : 
   - -t, --token generates the refresh token without performing export, useful to generate the token and then use it elsewhere on a headless server
   - -z, --zip indicates whether to zip the exported files automatically after export
