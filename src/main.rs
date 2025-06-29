use std::{fs::File, io::Read, path::Path, str::FromStr, sync::{Arc, Mutex}};
use chrono::{DateTime, Local, Utc};
use spotify::SpotifyAPI;
use templates::page;
use threadpool::ThreadPool;
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};

mod threadpool;
mod urldecode;
mod spotify;
mod templates;

struct Message {
    author: String,
    content: String,
    timestamp: DateTime<Utc>
}

struct App {
    pub spotify_api: SpotifyAPI,
    pub messages: Vec<Message>,
}

fn send_response<R>(request: Request, response: Response<R>) -> Result<(), ()>
where 
    R: Read
{
    request.respond(response)
        .map_err(|e| eprintln!("ERROR: couldn't respond to request: {e}"))
}

fn handle_static(request: Request) -> Result<(), ()> {
    let rel_path = request.url().trim_start_matches("/");
    if rel_path.contains("..") {
        return request.respond(Response::empty(404))
            .map_err(|e| eprintln!("ERROR: couldn't respond to request: {e}"))
    }
    
    let path = Path::new("./").join(rel_path);

    if !path.exists() || !path.is_file() {
        println!("{path:?}");
        return send_response(request, Response::from_string(templates::not_found())
            .with_status_code(StatusCode(404))
            .with_header(Header::from_str("Content-Type: text/html").unwrap())
        )
    }

    let file = File::open(&path).map_err(|e| eprintln!("ERROR: couldn't open file: {e}"))?;

    let content_type = match path.extension().and_then(|ext| ext.to_str()) {
        Some("css") => "text/css; charset=utf-8",
        Some("js")  => "application/javascript; charset=utf-8",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        _           => "application/octet-stream"
    };

    let response = Response::from_file(file)
        .with_header(Header::from_str(&format!("Content-Type: {content_type}")).unwrap())
        .with_header(Header::from_str("Cache-Control: public, max-age=86400").unwrap());
    
    send_response(request, response)
}
    
fn handle_fragment(mut request: Request, app: &mut App) -> Result<(), ()> {
    let method = request.method();
    let url = request.url();

    let (title, template) = match (method, url) {
        (Method::Get, "/" | "/home") => ("Home", templates::home()),
        (Method::Get, "/guestbook")  => ("Guestbook", templates::guestbook()),
        (Method::Get, "/projects")   => ("Projects", templates::projects()),
        (Method::Get, "/interests")  => ("Interests", templates::interests()),

        (Method::Get, "/now-playing")  => ("Now Playing",  templates::now_playing(app)),
        (Method::Get, "/current-time") => ("Current Time", templates::current_time()),
        (Method::Get, "/weather")      => ("Weather",      templates::weather()),
        (Method::Get, "/host-uptime")  => ("Host Uptime",  templates::host_uptime()),

        (Method::Get, "/guestbook/messages") => ("Guestbook Messages", templates::messages(&app.messages)),

        (Method::Post, "/guestbook") => ("Guestbook Messages", {
            let mut body = String::new();
            request.as_reader().read_to_string(&mut body).unwrap();

            let parsed = urldecode::parse_urlencoded(&body);

            if let (Some(author), Some(content)) = (parsed.get("author"), parsed.get("content")) {
                app.messages.push(Message {
                    author: author.to_string(),
                    content: content.to_string(),
                    timestamp: Local::now().into() 
                }); 
            } 

            templates::messages(&app.messages)
        }),
        _ => ("Not Found", templates::not_found())
    };

    let is_htmx = request.headers()
        .iter()
        .any(|h| h.field.equiv("HX-Request"));


    let body = if is_htmx {
        template
    } else {
        page(title, template)
    };

    let response = Response::from_string(body)
        .with_header(Header::from_str("Content-Type: text/html; charset=utf-8").unwrap());

    send_response(request, response)
}

fn handle_request(request: Request, app: &mut App) -> Result<(), ()> {
    match request.url() {
        url if url.starts_with("/static") => handle_static(request),
        _                                 => handle_fragment(request, app),
    }
}

fn main() -> Result<(), ()> {
    let pool = ThreadPool::new(4);

    let app = Arc::new(Mutex::new(App {
        spotify_api: SpotifyAPI::new(),
        messages: Vec::new(),
    }));

    let address = "127.0.0.1:3000";
    let server = Server::http(address)
        .map_err(|e| eprintln!("ERROR: couldn't start server: {e}"))?;

    println!("INFO: started server at {address}");

    for request in server.incoming_requests() {
        let app = Arc::clone(&app);
        pool.execute(move || {
            let mut app = app.lock().unwrap();
            let _ = handle_request(request, &mut *app);
        });
    }

    Ok(())
}
