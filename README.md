# SpotifyExporter
A small script to pull your data from Spotify and save it locally as JSON files.
The reason for this is that overtime tracks that you save in Spotify could become unavailable in your region.
Currently when this happens you can still see them in your liked list, but they are greyed out.
Notably you can't see these anywhere else in the app(eg when searching for the exact etc.).
This saves the data locally so that should Spotify deprecate support for liked tracks that are no longer available in your region, you still have record of them if you want to find them elsewhere.

# Output
Currently this script saves all user specific data from the API as JSON files.
The JSON structures are left mostly in tact as it is intended to be a simple dump processed seperately.
The data exported : 
   - Liked Songs
   - Liked Albums
   - Liked Audiobooks
   - Liked Podcast Episodes
   - Followed and Created Playlists
   - Liked Shows
   - Followed Artists
Output can be found in the output folder in seperate JSON files with dates in the filename.

# Setup
This is a relatively simple script so the setup should be quick and easy.
This assumes you already have Rust setup.
1. Create an app on the Spotify Web API as instructed here : https://developer.spotify.com/documentation/web-api
   - Take note of the client ID generated
   - Set your redirect URI to http://localhost:8000/callback
2. Rename properties.default.toml to properties.toml
3. After renaming replace "clientId" with the client ID from step 1.
4. Build using cargo build
5. Run using cargo run
