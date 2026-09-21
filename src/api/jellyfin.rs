use std::{
    collections::{HashMap, HashSet},
    fmt,
    sync::OnceLock,
    time::Duration,
};

use serde::{de::DeserializeOwned, Deserialize};
use url::Url;

const PAGE_SIZE: usize = 5_000;

#[derive(Debug, Clone)]
pub struct Track {
    pub name: String,
    pub artist: String,
}

#[derive(Debug, Clone)]
pub struct Album {
    pub name: String,
    pub artist: String,
    pub playcount: u64,
}

#[derive(Debug, Clone)]
pub struct Artist {
    pub name: String,
    pub playcount: u64,
}

#[derive(Debug, Clone)]
pub struct MusicRankings {
    pub top_artists: Vec<Artist>,
    pub top_albums: Vec<Album>,
}

#[derive(Debug, Clone)]
pub struct UserStats {
    pub library_tracks: usize,
    pub library_artists: usize,
    pub library_albums: usize,
}

#[derive(Debug, Clone)]
pub struct MusicStats {
    pub top_tracks: Vec<Track>,
    pub user_stats: UserStats,
}

#[derive(Debug)]
pub enum JellyfinError {
    InvalidUrl(url::ParseError),
    Request(Box<ureq::Error>),
    InvalidResponse(std::io::Error),
    UserNotFound(String),
}

impl fmt::Display for JellyfinError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUrl(error) => write!(f, "invalid URL: {error}"),
            Self::Request(error) => write!(f, "request failed: {error}"),
            Self::InvalidResponse(error) => write!(f, "invalid response: {error}"),
            Self::UserNotFound(username) => write!(f, "user `{username}` was not found"),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct JellyfinUser {
    id: String,
    name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct JellyfinSession {
    #[serde(default)]
    user_id: String,
    #[serde(default)]
    now_playing_item: Option<JellyfinItem>,
    #[serde(default)]
    play_state: Option<JellyfinPlayState>,
    #[serde(default)]
    last_activity_date: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct JellyfinPlayState {
    #[serde(default)]
    is_paused: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct JellyfinItem {
    #[serde(default)]
    name: Option<String>,
    #[serde(rename = "Type", default)]
    item_type: Option<String>,
    #[serde(default)]
    album: Option<String>,
    #[serde(default)]
    album_artist: Option<String>,
    #[serde(default)]
    artists: Vec<String>,
    #[serde(default)]
    user_data: Option<JellyfinUserData>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct JellyfinUserData {
    #[serde(default)]
    play_count: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ItemsResponse {
    #[serde(default)]
    items: Vec<JellyfinItem>,
    #[serde(default)]
    total_record_count: usize,
}

pub struct JellyfinApi {
    base_url: Url,
    authorization: String,
    username: String,
    user_id: OnceLock<String>,
    agent: ureq::Agent,
}

impl JellyfinApi {
    pub fn new(
        base_url: Url,
        api_key: String,
        username: String,
        configured_user_id: Option<String>,
    ) -> Self {
        let authorization = format!(
            "MediaBrowser Client=\"personal-website\", Device=\"server\", DeviceId=\"personal-website\", Version=\"{}\", Token=\"{}\"",
            env!("CARGO_PKG_VERSION"),
            api_key
        );

        let user_id = OnceLock::new();
        if let Some(configured_user_id) = configured_user_id {
            let _ = user_id.set(configured_user_id);
        }

        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(5))
            .timeout_read(Duration::from_secs(15))
            .timeout_write(Duration::from_secs(5))
            .build();

        Self {
            base_url,
            authorization,
            username,
            user_id,
            agent,
        }
    }

    fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<T, JellyfinError> {
        let mut url = self
            .base_url
            .join(path)
            .map_err(JellyfinError::InvalidUrl)?;

        url.query_pairs_mut()
            .extend_pairs(query.iter().map(|(key, value)| (*key, value)));

        self.agent
            .get(url.as_str())
            .set("Authorization", &self.authorization)
            .call()
            .map_err(|error| JellyfinError::Request(Box::new(error)))?
            .into_json()
            .map_err(JellyfinError::InvalidResponse)
    }

    fn get_user_id(&self) -> Result<&str, JellyfinError> {
        if let Some(user_id) = self.user_id.get() {
            return Ok(user_id);
        }

        let users: Vec<JellyfinUser> = self.get("/Users", &[])?;
        let user_id = users
            .into_iter()
            .find(|user| user.name.eq_ignore_ascii_case(&self.username))
            .map(|user| user.id)
            .ok_or_else(|| JellyfinError::UserNotFound(self.username.clone()))?;

        let _ = self.user_id.set(user_id);
        Ok(self.user_id.get().expect("user id was just initialized"))
    }

    pub fn get_now_playing(&self) -> Result<Option<Track>, JellyfinError> {
        let user_id = normalize_id(self.get_user_id()?);
        let sessions: Vec<JellyfinSession> =
            self.get("/Sessions", &[("activeWithinSeconds", "300".to_string())])?;

        let session = sessions
            .into_iter()
            .filter(|session| normalize_id(&session.user_id) == user_id)
            .filter(|session| {
                session
                    .now_playing_item
                    .as_ref()
                    .and_then(|item| item.item_type.as_deref())
                    == Some("Audio")
            })
            .filter(|session| {
                !session
                    .play_state
                    .as_ref()
                    .is_some_and(|state| state.is_paused)
            })
            .max_by(|left, right| left.last_activity_date.cmp(&right.last_activity_date));

        Ok(session.and_then(|session| {
            let item = session.now_playing_item?;
            let artist = display_artist(&item);
            Some(Track {
                name: item.name.unwrap_or_else(|| "Unknown track".to_string()),
                artist,
            })
        }))
    }

    pub fn get_music_stats(&self) -> Result<MusicStats, JellyfinError> {
        let user_id = self.get_user_id()?.to_string();
        let tracks: ItemsResponse = self.get(
            "/Items",
            &[
                ("userId", user_id.clone()),
                ("recursive", "true".to_string()),
                ("includeItemTypes", "Audio".to_string()),
                ("sortBy", "PlayCount".to_string()),
                ("sortOrder", "Descending".to_string()),
                ("enableUserData", "false".to_string()),
                ("enableTotalRecordCount", "true".to_string()),
                ("enableImages", "false".to_string()),
                ("limit", "10".to_string()),
            ],
        )?;

        let library_artists = self.get_item_count(&user_id, "MusicArtist")?;
        let library_albums = self.get_item_count(&user_id, "MusicAlbum")?;

        let top_tracks = tracks
            .items
            .into_iter()
            .map(|item| {
                let artist = display_artist(&item);
                Track {
                    name: item.name.unwrap_or_else(|| "Unknown track".to_string()),
                    artist,
                }
            })
            .collect();

        Ok(MusicStats {
            top_tracks,
            user_stats: UserStats {
                library_tracks: tracks.total_record_count,
                library_artists,
                library_albums,
            },
        })
    }

    pub fn get_music_rankings(&self) -> Result<MusicRankings, JellyfinError> {
        let user_id = self.get_user_id()?.to_string();
        let mut start_index = 0;
        let mut items = Vec::new();

        loop {
            let page: ItemsResponse = self.get(
                "/Items",
                &[
                    ("userId", user_id.clone()),
                    ("recursive", "true".to_string()),
                    ("includeItemTypes", "Audio".to_string()),
                    ("sortBy", "PlayCount".to_string()),
                    ("sortOrder", "Descending".to_string()),
                    ("enableUserData", "true".to_string()),
                    ("enableTotalRecordCount", "true".to_string()),
                    ("enableImages", "false".to_string()),
                    ("startIndex", start_index.to_string()),
                    ("limit", PAGE_SIZE.to_string()),
                ],
            )?;

            let page_len = page.items.len();
            let has_unplayed = page
                .items
                .iter()
                .any(|item| item.user_data.as_ref().map_or(0, |data| data.play_count) == 0);
            items.extend(
                page.items.into_iter().take_while(|item| {
                    item.user_data.as_ref().map_or(0, |data| data.play_count) > 0
                }),
            );
            start_index += page_len;

            if has_unplayed || page_len == 0 || start_index >= page.total_record_count {
                break;
            }
        }

        Ok(aggregate_rankings(items))
    }

    fn get_item_count(&self, user_id: &str, item_type: &str) -> Result<usize, JellyfinError> {
        let response: ItemsResponse = self.get(
            "/Items",
            &[
                ("userId", user_id.to_string()),
                ("recursive", "true".to_string()),
                ("includeItemTypes", item_type.to_string()),
                ("enableUserData", "false".to_string()),
                ("enableTotalRecordCount", "true".to_string()),
                ("enableImages", "false".to_string()),
                ("limit", "1".to_string()),
            ],
        )?;

        Ok(response.total_record_count)
    }
}

fn aggregate_rankings(items: Vec<JellyfinItem>) -> MusicRankings {
    let mut artist_plays: HashMap<String, u64> = HashMap::new();
    let mut album_plays: HashMap<(String, String), u64> = HashMap::new();

    for item in items {
        let playcount = item.user_data.as_ref().map_or(0, |data| data.play_count);
        let display_artist = display_artist(&item);

        let artists: HashSet<String> = if item.artists.is_empty() {
            item.album_artist.clone().into_iter().collect()
        } else {
            item.artists.iter().cloned().collect()
        };
        for artist in artists {
            let current = artist_plays.entry(artist).or_default();
            *current = current.saturating_add(playcount);
        }

        if let Some(album) = item.album.filter(|album| !album.trim().is_empty()) {
            let album_artist = item
                .album_artist
                .filter(|artist| !artist.trim().is_empty())
                .unwrap_or(display_artist);
            let current = album_plays.entry((album_artist, album)).or_default();
            *current = current.saturating_add(playcount);
        }
    }

    let mut top_artists: Vec<Artist> = artist_plays
        .into_iter()
        .map(|(name, playcount)| Artist { name, playcount })
        .collect();
    top_artists.sort_by(|left, right| {
        right
            .playcount
            .cmp(&left.playcount)
            .then_with(|| left.name.cmp(&right.name))
    });
    top_artists.truncate(10);

    let mut top_albums: Vec<Album> = album_plays
        .into_iter()
        .map(|((artist, name), playcount)| Album {
            name,
            artist,
            playcount,
        })
        .collect();
    top_albums.sort_by(|left, right| {
        right
            .playcount
            .cmp(&left.playcount)
            .then_with(|| left.name.cmp(&right.name))
    });
    top_albums.truncate(10);

    MusicRankings {
        top_artists,
        top_albums,
    }
}

fn display_artist(item: &JellyfinItem) -> String {
    if !item.artists.is_empty() {
        item.artists.join(", ")
    } else if let Some(album_artist) = item.album_artist.as_deref() {
        album_artist.to_string()
    } else {
        "Unknown artist".to_string()
    }
}

fn normalize_id(id: &str) -> String {
    id.chars()
        .filter(|character| *character != '-')
        .flat_map(char::to_lowercase)
        .collect()
}
