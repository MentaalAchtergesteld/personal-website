use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use dotenv::dotenv;
use tiny_http::Server;

use crate::{
    api::{jellyfin::JellyfinApi, wttr::WttrApi},
    auth::AdminAuth,
    config::Config,
    db::MessageDb,
    models::load_projects,
    state::{App, JellyfinCache, WttrCache},
    util::{rate_limiter::RateLimiter, threadpool::ThreadPool},
};

mod api;
mod auth;
mod config;
mod db;
mod handlers;
mod models;
mod state;
mod ui;
mod util;

fn main() -> Result<(), ()> {
    dotenv().ok();

    let config = Config::from_env().map_err(|e| eprintln!("ERROR: Invalid configuration: {e}"))?;

    let admin_auth = config
        .admin
        .map(AdminAuth::new)
        .transpose()
        .map_err(|e| eprintln!("ERROR: Invalid admin configuration: {e}"))?;
    let admin_enabled = admin_auth.is_some();

    let server = Server::http(config.server_address)
        .map_err(|e| eprintln!("ERROR: Couldn't start server: {e}"))?;

    let app = Arc::new(App {
        admin_auth,
        wttr: WttrApi::new(),
        jellyfin: JellyfinApi::new(
            config.jellyfin.base_url,
            config.jellyfin.api_key,
            config.jellyfin.username,
            config.jellyfin.user_id,
        ),

        wttr_cache: WttrCache::new(),
        jellyfin_cache: JellyfinCache::new(),

        projects: load_projects("static/projects.toml")?,
        message_db: Arc::new(Mutex::new(MessageDb::new(config.database_path)?)),
        rate_limiter: Arc::new(Mutex::new(RateLimiter::new(Duration::from_secs(10)))),
        admin_login_rate_limiter: Arc::new(Mutex::new(RateLimiter::new(Duration::from_secs(3)))),
    });

    println!("Server listening on address {}", config.server_address);
    println!("Admin mode configured: {admin_enabled}");

    let pool = ThreadPool::new(16);

    let warm_app = Arc::clone(&app);
    pool.execute(move || handlers::warm_jellyfin_cache(&warm_app));

    for request in server.incoming_requests() {
        let app = Arc::clone(&app);

        pool.execute(move || {
            let _ = handlers::handle_request(request, app)
                .map_err(|_| eprintln!("ERROR: Couldn't handle request."));
        });
    }
    Ok(())
}
