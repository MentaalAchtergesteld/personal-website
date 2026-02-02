use std::{env, sync::OnceLock};

use serde_json::Value;

static URL: OnceLock<String> = OnceLock::new();

fn build_url() -> String {
    dotenv::dotenv().ok();
    let api_key = env::var("LASTFM_KEY").unwrap();

    format!("https://ws.audioscrobbler.com/2.0/?method=user.getrecenttracks&user=gravitowl&api_key={}&format=json&limit=1", api_key)
}

pub fn get_now_playing() -> Result<String, ()> {
    let response = ureq::get(URL.get_or_init(build_url)).call().map_err(|_| ())?;
    let body = response.into_body().read_to_string()
        .map_err(|e| eprintln!("ERROR: couldn't read response body to string: {e}"))?;

    let json: Value = serde_json::from_str(&body).map_err(|_| ())?;

    let track = json["recenttracks"]["track"][0].as_object().ok_or(())?;

    let is_now_playing = track
        .get("@attr")
        .and_then(|attr| attr.get("nowplaying"))
        .and_then(|np| np.as_str())
        .map(|s| s == "true")
        .unwrap_or(false);

    if !is_now_playing {
        return Ok("☹ Nothing playing right now ☹".to_string());
    }

    let track_name = track.get("name").and_then(|v| v.as_str()).unwrap_or("No Track");
    let artist_name = track
        .get("artist")
        .and_then(|a| a.get("#text"))
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown Artist");

    Ok(format!("♫ Now Playing: {} by {} ♫", track_name, artist_name))
}
