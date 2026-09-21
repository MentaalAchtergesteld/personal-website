use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::{
    api::{
        jellyfin::{JellyfinApi, MusicRankings, MusicStats, Track},
        wttr::WttrApi,
    },
    auth::AdminAuth,
    db::MessageDb,
    models::Project,
    util::{cache::Cache, rate_limiter::RateLimiter},
};

#[derive(Clone)]
pub struct JellyfinCache {
    pub now_playing: Cache<Option<Track>>,
    pub music_stats: Cache<MusicStats>,
    pub music_rankings: Cache<MusicRankings>,
}

impl JellyfinCache {
    pub fn new() -> Self {
        Self {
            now_playing: Cache::new(Duration::from_secs(15)),
            music_stats: Cache::new(Duration::from_hours(1)),
            music_rankings: Cache::new(Duration::from_hours(24 * 7)),
        }
    }
}

#[derive(Clone)]
pub struct WttrCache {
    pub weather: Cache<String>,
}

impl WttrCache {
    pub fn new() -> Self {
        Self {
            weather: Cache::new(Duration::from_mins(15)),
        }
    }
}

pub struct App {
    pub admin_auth: Option<AdminAuth>,
    pub wttr: WttrApi,
    pub jellyfin: JellyfinApi,

    pub wttr_cache: WttrCache,
    pub jellyfin_cache: JellyfinCache,

    pub projects: Vec<Project>,
    pub message_db: Arc<Mutex<MessageDb>>,
    pub rate_limiter: Arc<Mutex<RateLimiter>>,
    pub admin_login_rate_limiter: Arc<Mutex<RateLimiter>>,
}
