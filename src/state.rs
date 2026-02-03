use std::time::Duration;

use crate::{api::{lastfm::{Album, Artist, LastfmApi, Track, UserStats}, wttr::WttrApi}, util::cache::Cache};

pub struct LastfmCache {
    pub now_playing: Cache<Option<Track>>,
    pub top_artists: Cache<Vec<Artist>>,
    pub top_tracks: Cache<Vec<Track>>,
    pub top_albums: Cache<Vec<Album>>,
    pub user_stats: Cache<UserStats>,
}

impl LastfmCache {
    pub fn new() -> Self {
        Self {
            now_playing: Cache::new(Duration::from_mins(1)),
            top_artists: Cache::new(Duration::from_hours(60)),
            top_tracks: Cache::new(Duration::from_hours(1)),
            top_albums: Cache::new(Duration::from_hours(1)),
            user_stats: Cache::new(Duration::from_mins(5)),
        }
    }
}

pub struct WttrCache {
    pub weather: Cache<String>,
}

impl WttrCache {
    pub fn new() -> Self {
        Self { weather: Cache::new(Duration::from_mins(15)) }
    }
}

pub struct App {
    pub wttr: WttrApi,
    pub lastfm: LastfmApi,

    pub wttr_cache: WttrCache,
    pub lastfm_cache: LastfmCache,
}
