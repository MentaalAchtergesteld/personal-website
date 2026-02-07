use std::{env, sync::Arc};

use dotenv::dotenv;
use tiny_http::Server;

use crate::{api::{lastfm::LastfmApi, wttr::WttrApi}, models::load_projects, state::{App, LastfmCache, WttrCache}, threadpool::ThreadPool};

mod api;
mod ui;
mod handlers;
mod models;
mod state;
mod threadpool;
mod util;

fn main() -> Result<(), ()> {
    dotenv().ok();


    let address = env::var("SERVER_ADDRESS")
        .map_err(|e| eprintln!("ERROR: Couldn't get server address: {e}"))?;
    let server = Server::http(&address)
        .map_err(|e| eprintln!("ERROR: Couldn't start server: {e}"))?;

    let lastfm_key = env::var("LASTFM_KEY")
        .map_err(|e| eprintln!("ERROR: Couldn't get lastfm key: {e}"))?;

    let app = Arc::new(App {
        wttr: WttrApi::new(),
        lastfm: LastfmApi::new(lastfm_key, "gravitowl".into()),

        wttr_cache: WttrCache::new(),
        lastfm_cache: LastfmCache::new(),

        projects: load_projects("static/projects.toml")?
    });

    println!("Server listening on address {address}");

    let pool = ThreadPool::new(16);

    for request in server.incoming_requests() {
        let app = Arc::clone(&app);

        pool.execute(move || {
            let _ = handlers::handle_request(request, app)
                .map_err(|_| eprintln!("ERROR: Couldn't handle request."));
        });
    }
    Ok(())  
}
