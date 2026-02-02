use std::{env, sync::{Arc, Mutex}};

use dotenv::dotenv;
use tiny_http::Server;

use crate::{state::App, threadpool::ThreadPool};

mod api;
mod ui;
mod handlers;
mod models;
mod state;
mod threadpool;
mod utils;

fn main() -> Result<(), ()> {
    dotenv().ok();


    let address = env::var("SERVER_ADDRESS")
        .map_err(|e| eprintln!("ERROR: Couldn't get server address: {e}"))?;
    let server = Server::http(&address)
        .map_err(|e| eprintln!("ERROR: Couldn't start server: {e}"))?;

    let pool = ThreadPool::new(16);

    let app = Arc::new(Mutex::new(App {}));

    println!("Server listening on address {address}");

    for request in server.incoming_requests() {
        let app = Arc::clone(&app);

        pool.execute(move || {
            let _ = handlers::handle_request(request, app)
                .map_err(|_| eprintln!("ERROR: Couldn't handle request."));
        });
    }
    Ok(())  
}
