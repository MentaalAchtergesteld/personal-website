use crate::api::{lastfm::LastfmApi, wttr::WttrApi};

pub struct App {
    pub wttr: WttrApi,
    pub lastfm: LastfmApi
}
