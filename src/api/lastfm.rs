use std::{time::Duration};
use serde::{Deserialize, Deserializer, de::DeserializeOwned};

fn from_string_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where D: Deserializer<'de>
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Ok(s == "true")
}

#[derive(Deserialize, Debug)]
struct LfmImage {
    #[serde(rename="#text")]
     url: String,
     size: String
}

#[derive(Deserialize)]
struct RecentTracksRoot {
     recenttracks: RecentTracks
}

#[derive(Deserialize)]
struct RecentTracks {
    #[serde(rename="track")]
    tracks: Vec<LfmRecentTrack>,
    #[serde(rename="@attr")]
    attr: LfmPageAttr,
}

#[derive(Deserialize)]
struct LfmPageAttr {
     total: String,
     user: String
}

#[derive(Deserialize)]
struct LfmRecentTrack {
    name: String,
    url: String,
    artist: LfmTextObj,
    album: LfmTextObj,
    #[serde(rename="image")]
    images: Vec<LfmImage>,
    #[serde(rename="@attr")]
    attr: Option<LfmTrackAttr>
}

#[derive(Deserialize)]
struct LfmTrackAttr {
    #[serde(rename="nowplaying", deserialize_with="from_string_bool")]
    now_playing: bool,
}

#[derive(Deserialize)]
struct LfmTextObj {
    #[serde(rename="#text")]
    text: String
}


#[derive(Deserialize)]
struct TopAlbumsRoot {
    topalbums: TopAlbums
}

#[derive(Deserialize)]
struct TopAlbums {
    #[serde(rename="album")]
    albums: Vec<LfmTopAlbum>,
    #[serde(rename="@attr")]
    attr: LfmPageAttr
}

#[derive(Deserialize)]
struct LfmTopAlbum {
    name: String,
    url: String,
    playcount: String,
    artist: LfmSimpleArtist,
    #[serde(rename="image")]
    images: Vec<LfmImage>
}

#[derive(Deserialize)]
struct LfmSimpleArtist {
    name: String,
    url: String
}

#[derive(Deserialize)]
struct TopArtistsRoot {
    topartists: TopArtists
}

#[derive(Deserialize)]
struct TopArtists {
    #[serde(rename="artist")]
    artists: Vec<LfmTopArtist>,
    #[serde(rename="@attr")]
    attr: LfmPageAttr
}

#[derive(Deserialize)]
struct LfmTopArtist {
    name: String,
    url: String,
    playcount: String,
    #[serde(rename="image")]
    images: Vec<LfmImage>,
}

#[derive(Deserialize)]
struct TopTracksRoot {
    toptracks: TopTracks
}

#[derive(Deserialize)]
struct TopTracks {
    #[serde(rename="track")]
    tracks: Vec<LfmTopTrack>,
    #[serde(rename="@attr")]
    attr: LfmPageAttr
}

#[derive(Deserialize)]
struct LfmTopTrack {
    name: String,
    url: String,
    playcount: String,
    artist: LfmSimpleArtist,
    #[serde(rename="image")]
    images: Vec<LfmImage>
}

#[derive(Deserialize)]
struct UserInfoRoot {
    user: LfmUser
}

#[derive(Deserialize)]
struct LfmUser {
    name: String,
    playcount: String,
    artist_couint: String
}

// PUBLIC

#[derive(Debug, Clone)]
pub struct Track {
    pub name: String,
    pub artist: String,
    pub url: String,
    pub image_url: Option<String>,
    pub is_playing: bool,
    pub playcount: Option<u64>
}

#[derive(Debug, Clone)]
pub struct Album {
    pub name: String,
    pub artist: String,
    pub url: String,
    pub image_url: Option<String>,
    pub playcount: u64
}

#[derive(Debug, Clone)]
pub struct Artist {
    pub name: String,
    pub url: String,
    pub image_url: Option<String>,
    pub playcount: u64
}

#[derive(Debug, Clone)]
pub struct UserStats {
    pub total_scrobbles: u64,
    pub total_artists: u64,
    pub total_albums: u64,
    pub total_tracks: u64
}

pub struct LastfmApi {
    api_key: String,
    username: String,
    agent: ureq::Agent
}

impl LastfmApi {
     pub fn new(api_key: String, username: String) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout_read(Duration::from_secs(5))
            .timeout_write(Duration::from_secs(5))
            .build();

        Self { api_key, username, agent }
    }

    fn get<T: DeserializeOwned>(&self, method: &str, params: &str) -> Option<T> {
        let url = format!(
            "https://ws.audioscrobbler.com/2.0/?method={}&user={}&api_key={}&format=json&{}",
            method, self.username, self.api_key, params
        );

        let resp = self.agent.get(&url).call().ok()?;
        resp.into_json()
            .map_err(|e| eprintln!("ERROR: LastFM Deserialization issue [{method}]: {e}"))
            .ok()
    } 

    pub fn get_recent_tracks(&self, limit: usize) -> Vec<Track> {
        let res: Option<RecentTracksRoot> = self.get("user.getRecentTracks", &format!("limit={limit}"));

        match res {
            Some(root) => root.recenttracks.tracks.into_iter().map(|t| Track {
                name: t.name,
                artist: t.artist.text,
                url: t.url,
                image_url: t.images.last().map(|i| i.url.clone()),
                is_playing: t.attr.map(|a| a.now_playing).unwrap_or(false),
                playcount: None
            }).collect(),
            None => Vec::new()
        }
    }

    pub fn get_now_playing(&self) -> Option<Track> {
        let tracks = self.get_recent_tracks(1);
        let track = tracks.first()?;
        if track.is_playing {
            Some(track.clone()) 
        } else {
            None
        } 
    }

    pub fn get_top_albums(&self, limit: usize, period: &str) -> Vec<Album> {
        let params = format!("limit={limit}&period={period}");
        let res: Option<TopAlbumsRoot> = self.get("user.getTopAlbums", &params);

        match res {
            Some(root) => root.topalbums.albums.into_iter().map(|a| Album {
                name: a.name,
                artist: a.artist.name,
                url: a.url,
                image_url: a.images.last().map(|i| i.url.clone()),
                playcount: a.playcount.parse().unwrap_or(0),
            }).collect(),
            None => Vec::new(),
        }
    }

    pub fn get_top_artists(&self, limit: usize, period: &str) -> Vec<Artist> {
        let params = format!("limit={limit}&period={period}");
        let res: Option<TopArtistsRoot> = self.get("user.getTopArtists", &params);

        match res {
            Some(root) => root.topartists.artists.into_iter().map(|a| Artist {
                name: a.name,
                url: a.url,
                image_url: a.images.last().map(|i| i.url.clone()),
                playcount: a.playcount.parse().unwrap_or(0),
            }).collect(),
            None => Vec::new(),
        }
    }

    pub fn get_top_tracks(&self, limit: usize, period: &str) -> Vec<Track> {
        let params = format!("limit={limit}&period={period}");
        let res: Option<TopTracksRoot> = self.get("user.getTopTracks", &params);

        match res {
            Some(root) => root.toptracks.tracks.into_iter().map(|t| Track {
                name: t.name,
                artist: t.artist.name,
                url: t.url,
                image_url: t.images.last().map(|i| i.url.clone()),
                is_playing: false,
                playcount: Some(t.playcount.parse().unwrap_or(0)),
            }).collect(),
            None => Vec::new(),
        }
    }

    pub fn get_user_stats(&self) -> Option<UserStats> {
        let recent: RecentTracksRoot = self.get("user.getRecenttracks", "limit=1")?;
        let total_scrobbles = recent.recenttracks.attr.total.parse().unwrap_or(0);

        let artists: TopArtistsRoot = self.get("user.getTopArtists", "limit=1")?;
        let total_artists = artists.topartists.attr.total.parse().unwrap_or(0);

        let albums: TopAlbumsRoot = self.get("user.getTopAlbums", "limit=1")?;
        let total_albums = albums.topalbums.attr.total.parse().unwrap_or(0);

        let tracks: TopTracksRoot = self.get("user.getTopTracks", "limit=1")?;
        let total_tracks = tracks.toptracks.attr.total.parse().unwrap_or(0);

        Some(UserStats {
            total_scrobbles,
            total_artists,
            total_albums,
            total_tracks
        })
    }
}
