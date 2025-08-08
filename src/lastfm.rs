use serde_json::Value;

const URL: &str = "https://ws.audioscrobbler.com/2.0/?method=user.getrecenttracks&user=gravitowl&api_key=2021f94a656700feeab64b3d433f095e&format=json&limit=1";

pub fn get_now_playing() -> Result<String, ()> {
    let response = ureq::get(URL).call().map_err(|_| ())?;
    let body = response.into_body().read_to_string()
        .map_err(|e| eprintln!("ERROR: couldn't read response body to string: {e}"))?;

    let json: Value = serde_json::from_str(&body).map_err(|_| ())?;

    // Pak de eerste track uit recenttracks.track array
    let track = json["recenttracks"]["track"][0].as_object().ok_or(())?;

    // Check if nowplaying attribute == "true"
    let is_now_playing = track
        .get("@attr")
        .and_then(|attr| attr.get("nowplaying"))
        .and_then(|np| np.as_str())
        .map(|s| s == "true")
        .unwrap_or(false);

    if !is_now_playing {
        return Ok("☹ Nothing playing right now ☹".to_string());
    }

    // Pak naam van track en artiest
    let track_name = track.get("name").and_then(|v| v.as_str()).unwrap_or("No Track");
    let artist_name = track
        .get("artist")
        .and_then(|a| a.get("#text"))
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown Artist");

    Ok(format!("♫ Now Playing: {} by {} ♫", track_name, artist_name))
}
