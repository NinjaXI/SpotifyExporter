import argparse
import json
import os
import sys
import tomllib
import zipfile
from datetime import datetime

from spotify.spotify_client import SpotifyClient

VERSION = "1.2.0"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(prog="spotify-exporter", description="Exports all your saved data from Spotify")
    parser.add_argument("-V", "--version", action="version", version=f"spotify-exporter {VERSION}")
    parser.add_argument("-t", "--token", action="store_true",
                        help="only retrieve refresh token to be used for authorization code flow, no exporting performed")
    parser.add_argument("-z", "--zip", action="store_true", help="zip exported files")
    return parser.parse_args()


def today() -> str:
    return datetime.now().strftime("%Y%m%d")


def write_json(path: str, data: dict):
    # compact with sorted keys, matching serde_json::to_string output
    with open(path, "w", encoding="utf-8", newline="") as json_file:
        json_file.write(json.dumps(data, separators=(",", ":"), sort_keys=True, ensure_ascii=False))


def print_progress(text: str):
    print(text, end="", flush=True)


def main():
    args = parse_args()

    with open("properties.toml", "rb") as properties_file:
        properties = tomllib.load(properties_file)
    spotify_client = SpotifyClient(str(properties["oauth_flow_type"]), str(properties["spotify_client_id"]), str(properties["spotify_client_secret"]))
    spotify_client.get_access_token()

    if args.token:
        print("Token retrieved and saved, please see token.txt", flush=True)
        return

    if not os.path.exists("output"):
        os.mkdir("output")

    export_saved_tracks(spotify_client)
    export_saved_albums(spotify_client)
    export_saved_audiobooks(spotify_client)
    export_saved_episodes(spotify_client)
    export_user_playlists(spotify_client)
    export_saved_shows(spotify_client)
    export_followed_artists(spotify_client)

    if args.zip:
        zip_exported_json()


def export_saved_tracks(spotify_client: SpotifyClient):
    print("Exporting saved tracks")
    print_progress("\rProcessing 0%")

    # retrieve first 50 tracks
    tracks_vector = []
    spotify_track_response = spotify_client.get_saved_tracks(0, 50)
    tracks_vector.extend(spotify_track_response["items"])

    # keep retrieving tracks until our count = total in spotify response
    while len(tracks_vector) < spotify_track_response["total"]:
        spotify_track_response = spotify_client.get_saved_tracks(len(tracks_vector), 50)
        tracks_vector.extend(spotify_track_response["items"])

        percentage = (len(tracks_vector) / spotify_track_response["total"]) * 100.0
        print_progress(f"\rProcessing {percentage:.0f}%")

    # save tracks as json struct to file
    write_json(f"output/tracks_{today()}.json", {"tracks": tracks_vector})

    print_progress("\rProcessing 100%\n")


def export_saved_albums(spotify_client: SpotifyClient):
    print("Exporting saved albums")
    print_progress("\rProcessing 0%")

    # retrieve first 50 albums
    albums_vector = []
    spotify_album_response = spotify_client.get_saved_albums(0, 50)
    albums_vector.extend(spotify_album_response["items"])

    # keep retrieving albums until our count = total in spotify response
    while len(albums_vector) < spotify_album_response["total"]:
        spotify_album_response = spotify_client.get_saved_albums(len(albums_vector), 50)
        albums_vector.extend(spotify_album_response["items"])

        percentage = (len(albums_vector) / spotify_album_response["total"]) * 100.0
        print_progress(f"\rProcessing {percentage:.0f}%")

    # save albums as json struct to file
    write_json(f"output/albums_{today()}.json", {"albums": albums_vector})

    print_progress("\rProcessing 100%\n")


def export_saved_audiobooks(spotify_client: SpotifyClient):
    print("Exporting saved audiobooks")
    print_progress("\rProcessing 0%")

    # retrieve first 50 audiobooks
    audiobooks_vector = []
    spotify_audiobook_response = spotify_client.get_saved_audiobooks(0, 50)
    audiobooks_vector.extend(spotify_audiobook_response["items"])

    # keep retrieving audiobooks until our count = total in spotify response
    while len(audiobooks_vector) < spotify_audiobook_response["total"]:
        # NOTE: kept from the Rust original, later pages are requested from the albums endpoint
        spotify_audiobook_response = spotify_client.get_saved_albums(len(audiobooks_vector), 50)
        audiobooks_vector.extend(spotify_audiobook_response["items"])

        percentage = (len(audiobooks_vector) / spotify_audiobook_response["total"]) * 100.0
        print_progress(f"\rProcessing {percentage:.0f}%")

    # save audiobooks as json struct to file
    write_json(f"output/audiobooks_{today()}.json", {"audiobooks": audiobooks_vector})

    print_progress("\rProcessing 100%\n")


def export_saved_episodes(spotify_client: SpotifyClient):
    print("Exporting saved episodes")
    print_progress("\rProcessing 0%")

    # retrieve first 50 episodes
    episodes_vector = []
    spotify_episode_response = spotify_client.get_saved_episodes(0, 50)
    episodes_vector.extend(spotify_episode_response["items"])

    # keep retrieving episodes until our count = total in spotify response
    while len(episodes_vector) < spotify_episode_response["total"]:
        # NOTE: kept from the Rust original, later pages are requested from the albums endpoint
        spotify_episode_response = spotify_client.get_saved_albums(len(episodes_vector), 50)
        episodes_vector.extend(spotify_episode_response["items"])

        percentage = (len(episodes_vector) / spotify_episode_response["total"]) * 100.0
        print_progress(f"\rProcessing {percentage:.0f}%")

    # save episodes as json struct to file
    write_json(f"output/episodes_{today()}.json", {"episodes": episodes_vector})

    print_progress("\rProcessing 100%\n")


def export_user_playlists(spotify_client: SpotifyClient):
    print("Exporting users owned or followed playlists")
    print_progress("\rProcessing 0%")

    # retrieve first 50 playlists
    playlists_vector = []
    spotify_playlist_response = spotify_client.get_owned_followed_playlists(0, 50)
    for playlist in spotify_playlist_response["items"]:
        # NOTE: kept from the Rust original, only the first 50 tracks of each playlist are retrieved
        spotify_track_response = spotify_client.get_playlist_tracks(playlist["id"], 0, 50)
        playlist["tracks"] = spotify_track_response["items"]

        playlists_vector.append(playlist)

    # keep retrieving playlists until our count = total in spotify response
    while len(playlists_vector) < spotify_playlist_response["total"]:
        spotify_playlist_response = spotify_client.get_owned_followed_playlists(len(playlists_vector), 50)
        for playlist in spotify_playlist_response["items"]:
            spotify_track_response = spotify_client.get_playlist_tracks(playlist["id"], 0, 50)
            playlist["tracks"] = spotify_track_response["items"]

            playlists_vector.append(playlist)

        percentage = (len(playlists_vector) / spotify_playlist_response["total"]) * 100.0
        print_progress(f"\rProcessing {percentage:.0f}%")

    # save playlists as json struct to file
    write_json(f"output/playlists_{today()}.json", {"playlists": playlists_vector})

    print_progress("\rProcessing 100%\n")


def export_saved_shows(spotify_client: SpotifyClient):
    print("Exporting saved shows")
    print_progress("\rProcessing 0%")

    # retrieve first 50 shows
    shows_vector = []
    # NOTE: kept from the Rust original, the first page is requested from the episodes endpoint
    spotify_show_response = spotify_client.get_saved_episodes(0, 50)
    shows_vector.extend(spotify_show_response["items"])

    # keep retrieving shows until our count = total in spotify response
    while len(shows_vector) < spotify_show_response["total"]:
        spotify_show_response = spotify_client.get_saved_shows(len(shows_vector), 50)
        shows_vector.extend(spotify_show_response["items"])

        percentage = (len(shows_vector) / spotify_show_response["total"]) * 100.0
        print_progress(f"\rProcessing {percentage:.0f}%")

    # save shows as json struct to file
    write_json(f"output/shows_{today()}.json", {"shows": shows_vector})

    print_progress("\rProcessing 100%\n")


def export_followed_artists(spotify_client: SpotifyClient):
    print("Exporting followed artists")
    print_progress("\rProcessing 0%")

    # retrieve first 50 artists
    artists_vector = []
    spotify_artist_response = spotify_client.get_followed_artists("", 50)
    artists_vector.extend(spotify_artist_response["artists"]["items"])

    # keep retrieving artists until our count = total in spotify response
    after = spotify_artist_response["artists"]["cursors"]["after"]
    if not isinstance(after, str):
        # NOTE: kept from the Rust original (as_str().unwrap()), fails when there is no next page cursor
        raise TypeError("Expected a cursor string for followed artists")
    while len(artists_vector) < spotify_artist_response["artists"]["total"]:
        spotify_artist_response = spotify_client.get_followed_artists(after, 50)
        artists_vector.extend(spotify_artist_response["artists"]["items"])
        after = spotify_artist_response["artists"]["cursors"]["after"] or ""

        percentage = (len(artists_vector) / spotify_artist_response["artists"]["total"]) * 100.0
        print_progress(f"\rProcessing {percentage:.0f}%")

    # save artists as json struct to file
    write_json(f"output/artists_{today()}.json", {"artists": artists_vector})

    print_progress("\rProcessing 100%\n")


def zip_exported_json():
    print("Zipping exported files", flush=True)

    with zipfile.ZipFile(f"output/{today()}_exported.zip", "w", compression=zipfile.ZIP_DEFLATED) as zip_writer:
        for entry in os.scandir("output"):
            if entry.is_file() and entry.path.endswith(f"{today()}.json"):
                zip_writer.write(entry.path, arcname=entry.name)
                os.remove(entry.path)


if __name__ == "__main__":
    sys.exit(main())
