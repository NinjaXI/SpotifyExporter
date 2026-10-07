import base64
import hashlib
import json
import os
import secrets
import socket
import string
import sys
import time
import webbrowser

import requests

ALPHANUMERIC = string.ascii_letters + string.digits


def _random_alphanumeric(length: int) -> str:
    return "".join(secrets.choice(ALPHANUMERIC) for _ in range(length))


class SpotifyClient:

    def __init__(self, flow_type: str, spotify_client_id: str, spotify_client_secret: str):
        self.flow_type = flow_type
        self.spotify_client_id = spotify_client_id
        self.spotify_client_secret = spotify_client_secret
        self.code_verifier = ""
        self.code_challenge = ""
        self.token_refreshed = 0
        self.access_token = ""
        self.refresh_token = ""
        self.token_type = ""
        self.expires_in = 0
        self.client = requests.Session()

    def get_access_token(self) -> bool:
        """Get access token for Spotify API

        This will open a browser window from Spotify asking the user to grant the privelages required to this script.
        Once granted, Spotify will do a callback request which the script will catch and serve a callback html for.
        This html file will, using javascript, extract the query parameters and do a request back to this script so that we can extract the access token here in the backend.
        """
        has_token = False
        if self.flow_type == "code":
            if os.path.exists("token.txt"):
                with open("token.txt", encoding="utf-8", newline="") as token_file:
                    self.refresh_token = token_file.read()

                has_token = self._refresh_access_token_validity()

        if not has_token:
            # start TCP Listener that will be used to receive callback requests as part of OAuth flow
            listener = socket.create_server(("127.0.0.1", 8000))

            # generate random 16 length string to validate in implicit grant
            state = _random_alphanumeric(16)
            scope = "user-library-read user-read-playback-position playlist-read-private user-follow-read"

            authorization_url = f"https://accounts.spotify.com/authorize?response_type={self.flow_type}&client_id={self.spotify_client_id}&scope={scope}&redirect_uri=http://127.0.0.1:8000/callback&state={state}"
            if self.flow_type == "code":
                self._generate_code_challenge()
                authorization_url += f"&code_challenge_method=S256&code_challenge={self.code_challenge}"
            webbrowser.open(authorization_url)

            running = True
            while running:
                try:
                    stream, _ = listener.accept()
                except OSError as e:
                    print(f"Error: {e}", file=sys.stderr)
                    continue

                with stream:
                    buffer = stream.recv(1024)

                    request = buffer.decode("utf-8")

                    first_line = request.splitlines()[0]
                    url = first_line.split()[1]

                    # we only expect 2 calls here, either the callback from spotify, or a finalize call from our own html
                    if "finalizeAuthentication" in url:
                        # if it is the finalize call we extract the relevant details from the URL and finalize the oauth flow
                        if self.flow_type == "token":
                            self._finalize_implicit_grant(url, state)
                        else:
                            self._finalize_authorization_code(url, state)  # TODO authorization code with PKCE, maybe?

                        running = False
                    else:
                        # if its not the finalize call we assume its the callback from spotify and serve our callback html
                        self._serve_callback(stream)

            listener.close()

        return True

    def get_saved_tracks(self, offset: int, limit: int) -> dict:
        """Retrieve the saved tracks for the user

        offset - An int that specifies the offset in the list of saved tracks
        limit - An int specifying total number of tracks to return, 50 is max
        """
        return self._get(f"https://api.spotify.com/v1/me/tracks?offset={offset}&limit={limit}")

    def get_saved_albums(self, offset: int, limit: int) -> dict:
        """Retrieve the saved albums for the user

        offset - An int that specifies the offset in the list of saved albums
        limit - An int specifying total number of albums to return, 50 is max
        """
        return self._get(f"https://api.spotify.com/v1/me/albums?offset={offset}&limit={limit}")

    def get_saved_audiobooks(self, offset: int, limit: int) -> dict:
        """Retrieve the saved audiobooks for the user

        offset - An int that specifies the offset in the list of saved audiobooks
        limit - An int specifying total number of audiobooks to return, 50 is max
        """
        return self._get(f"https://api.spotify.com/v1/me/audiobooks?offset={offset}&limit={limit}")

    def get_saved_episodes(self, offset: int, limit: int) -> dict:
        """Retrieve the saved episodes for the user

        offset - An int that specifies the offset in the list of saved episodes
        limit - An int specifying total number of episodes to return, 50 is max
        """
        return self._get(f"https://api.spotify.com/v1/me/episodes?offset={offset}&limit={limit}")

    def get_owned_followed_playlists(self, offset: int, limit: int) -> dict:
        """Retrieve the owned or followed playlists for the user

        offset - An int that specifies the offset in the list of playlists
        limit - An int specifying total number of playlists to return, 50 is max
        """
        return self._get(f"https://api.spotify.com/v1/me/playlists?offset={offset}&limit={limit}")

    def get_playlist_tracks(self, playlist_id: str, offset: int, limit: int) -> dict:
        """Retrieve the tracks of the playlist for the given playlist id

        playlist_id - The id of the playlist to retrieve tracks for
        offset - An int that specifies the offset in the list of tracks
        limit - An int specifying total number of tracks to return, 50 is max
        """
        fields = "items(added_by.id,added_at,track(id,name,album(album_type,name,release_date,artists(id,name)),artists(id,name)))"  # the fields specifier for track.album.artists has no affect
        return self._get(f"https://api.spotify.com/v1/playlists/{playlist_id}/tracks?fields={fields}&offset={offset}&limit={limit}")

    def get_saved_shows(self, offset: int, limit: int) -> dict:
        """Retrieve the saved shows for the user

        offset - An int that specifies the offset in the list of saved shows
        limit - An int specifying total number of shows to return, 50 is max
        """
        return self._get(f"https://api.spotify.com/v1/me/shows?offset={offset}&limit={limit}")

    def get_followed_artists(self, after: str, limit: int) -> dict:
        """Retrieve the followed artists for the user

        after - The cursor (last artist id) to continue from, empty for the first page
        limit - An int specifying total number of artists to return, 50 is max
        """
        if after == "":
            url = f"https://api.spotify.com/v1/me/following?type=artist&limit={limit}"
        else:
            url = f"https://api.spotify.com/v1/me/following?type=artist&after={after}&limit={limit}"

        return self._get(url)

    def _get(self, url: str) -> dict:
        """Performs an authorized GET request against the Spotify API and parses the JSON response"""
        if not self._refresh_access_token_validity():
            raise RuntimeError("No valid token")

        get_response = self.client.get(url, headers={"Authorization": f"{self.token_type} {self.access_token}"})
        return json.loads(get_response.text)

    def _serve_callback(self, stream: socket.socket):
        """Serves the html file in src/html/callback.html as response on the TCP stream

        stream - The TCP stream to serve the response on
        """
        try:
            with open("src/html/callback.html", encoding="utf-8") as html_file:
                content = html_file.read()
        except OSError:
            content = "Failed to read the HTML file"

        response = f"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n{content}"

        stream.sendall(response.encode("utf-8"))

    def _finalize_implicit_grant(self, url: str, state: str):
        """Extracts the access_token and other properties for the Spotify API from the url

        url - The URL to extract the query paramters from
        state - State string provided to Spotify in initial request that must match
        """
        query_params = url.split("?")[1].split("&")

        for param in query_params:
            param_arr = param.split("=")
            match param_arr[0]:
                case "access_token":
                    self.access_token = param_arr[1]
                case "token_type":
                    self.token_type = param_arr[1]
                case "expires_in":
                    self.expires_in = int(param_arr[1])
                case "state":
                    if state != param_arr[1]:
                        raise RuntimeError("State does not match")
                case _:
                    pass  # dont care

    def _finalize_authorization_code(self, url: str, state: str) -> bool:
        """Extracts the code for the Spotify API from the url
        Then continues the OAuth2.0 Authorization Code flow by using the code to request an access token

        url - The URL to extract the query paramters from
        state - State string provided to Spotify in initial request that must match
        """
        query_params = url.split("?")[1].split("&")

        for param in query_params:
            param_arr = param.split("=")
            match param_arr[0]:
                case "code":
                    authorization_code = param_arr[1]
                    form_params = {
                        "grant_type": "authorization_code",
                        "code": authorization_code,
                        "redirect_uri": "http://127.0.0.1:8000/callback",
                        "client_id": self.spotify_client_id,
                        "code_verifier": self.code_verifier,
                    }

                    access_token_url = "https://accounts.spotify.com/api/token"
                    access_token_response = self.client.post(access_token_url,
                                                             headers={"Content-Type": "application/x-www-form-urlencoded"},
                                                             data=form_params)
                    access_token_response_json = json.loads(access_token_response.text)
                    self.access_token = access_token_response_json["access_token"]
                    self.refresh_token = access_token_response_json["refresh_token"]
                    self.token_type = access_token_response_json["token_type"]
                    self.expires_in = access_token_response_json["expires_in"]
                    self.token_refreshed = int(time.time())

                    with open("token.txt", "w", encoding="utf-8", newline="") as token_file:
                        token_file.write(self.refresh_token)
                case "state":
                    if state != param_arr[1]:
                        raise RuntimeError("State does not match")
                case _:
                    pass  # dont care

        return True

    def _refresh_access_token_validity(self) -> bool:
        """Uses the stored refresh token to refresh the access token if it has expired or has not been retrieved yet.

        Returns True if a valid token has been retrieved
        """
        # only refresh token for authorization code flow
        if self.flow_type != "code":
            return True

        now_secs = int(time.time())
        if self.token_refreshed == 0 or (self.token_refreshed + self.expires_in) < (now_secs - 300):
            sys.stdout.flush()
            form_params = {
                "grant_type": "refresh_token",
                "refresh_token": self.refresh_token,
                "client_id": self.spotify_client_id,
            }

            refresh_token_url = "https://accounts.spotify.com/api/token"
            auth_header = "Basic " + base64.b64encode(f"{self.spotify_client_id}:{self.spotify_client_secret}".encode("utf-8")).decode("ascii")
            access_token_response = self.client.post(refresh_token_url,
                                                     headers={"Authorization": auth_header,
                                                              "Content-Type": "application/x-www-form-urlencoded"},
                                                     data=form_params)
            access_token_response_json = json.loads(access_token_response.text)
            self.access_token = access_token_response_json["access_token"]
            self.token_type = access_token_response_json["token_type"]
            self.expires_in = access_token_response_json["expires_in"]
            self.token_refreshed = int(time.time())

            if access_token_response_json.get("refresh_token") is not None:
                self.refresh_token = access_token_response_json["refresh_token"]

                with open("token.txt", "w", encoding="utf-8", newline="") as token_file:
                    token_file.write(self.refresh_token)

        return True

    def _generate_code_challenge(self):
        self.code_verifier = _random_alphanumeric(128)
        code_verifier_hashed = hashlib.sha256(self.code_verifier.encode("utf-8")).digest()
        self.code_challenge = base64.b64encode(code_verifier_hashed).decode("ascii").replace("/", "_").replace("+", "-").replace("=", "")
