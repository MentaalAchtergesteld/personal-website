use std::{env, time::{Duration, Instant}};

use base64::{engine, Engine};
use serde_json::Value;

pub struct SpotifyAPI {
    client_id: String,
    client_secret: String,
    refresh_token: String,
    access_token: Option<String>,
    token_expiry: Option<Instant>
}

impl SpotifyAPI {
    pub fn new() -> Self {
        dotenv::dotenv().ok();

        Self {
            client_id: env::var("SPOTIFY_ID").unwrap(),
            client_secret: env::var("SPOTIFY_SECRET").unwrap(),
            refresh_token: env::var("SPOTIFY_REFRESH").unwrap(),
            access_token: None,
            token_expiry: None
        }
    }

    pub fn refresh_access_token(&mut self) -> Result<(), ()> {
        let basic_auth = engine::general_purpose::STANDARD.encode(format!("{}:{}", self.client_id, self.client_secret));
        let response = ureq::post("https://accounts.spotify.com/api/token")
            .header("Authorization", &format!("Basic {}", basic_auth))
            .send_form([
                ("grant_type", "refresh_token"),
                ("refresh_token", &self.refresh_token)
            ])
            .map_err(|e| eprintln!("ERROR: couldn't refresh access token: {e}"))?
            .into_body()
            .read_to_string()
            .map_err(|e| eprintln!("ERROR: couldn't read response body to string: {e}"))?;

        let json: Value = serde_json::from_str(&response)
            .map_err(|e| eprintln!("ERROR: couldn't parse response into json: {e}"))?;

        let token = json["access_token"]
            .as_str()
            .ok_or("Missing access_token")
            .map_err(|e| eprintln!("ERROR: couldn't get access token: {e}"))?
            .to_string();

        let expires_in = json["expires_in"].as_u64().unwrap_or(3600);

        self.access_token = Some(token);
        self.token_expiry = Some(Instant::now() + Duration::from_secs(expires_in));

        Ok(())
    }

    fn get_access_token(&mut self) -> Result<&str, ()> {
        let needs_refresh = match self.token_expiry {
            Some(expiry) => Instant::now() >= expiry,
            None => true,
        };

        if needs_refresh { self.refresh_access_token()? };

        Ok(self.access_token.as_ref().unwrap())
    }

    pub fn get_now_playing(&mut self) -> Result<String, ()> {
        let access_token = self.get_access_token()?;
        let response = ureq::get("https://api.spotify.com/v1/me/player/currently-playing")
            .header("Authorization", &format!("Bearer {}", access_token))
            .call()
            .map_err(|e| eprintln!("ERROR: couldn't get currently playing: {e}"))?;

        if response.status() == 204 { return Ok("☹ Nothing playing right now ☹".to_string()) }

        let json_string = response.into_body().read_to_string()
            .map_err(|e| eprintln!("ERROR: couldn't read response body to string: {e}"))?;
        let json: Value = serde_json::from_str(&json_string)
            .map_err(|e| eprintln!("ERROR: couldn't parse response into json: {e}"))?;

        if !json["is_playing"].as_bool().unwrap_or(false) { return Ok("☹ Nothing playing right now ☹".to_string()) }

        let track_name = json["item"]["name"].as_str().unwrap_or("No Track");
        let artists = json["item"]["artists"]
            .as_array()
            .map(|arr| arr.iter()
                .filter_map(|a| a["name"].as_str())
                .collect::<Vec<_>>()
                .join(", "))
            .unwrap_or_default();

        Ok(format!("♫ Now Playing: {} by {} ♫", track_name, artists))
    }
}
