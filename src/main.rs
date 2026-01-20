use std::{fs::File, io::Read, net::IpAddr, path::Path, str::FromStr, sync::{Arc, Mutex, RwLock}, time::Duration};
use chrono::{DateTime, Local, Utc};
use maud::html;
use projects::{load_projects, Project};
use ratelimiter::RateLimiter;
use rusqlite::Connection;
use templates::{get_messages, page};
use threadpool::ThreadPool;
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};

mod threadpool;
mod ratelimiter;
mod urldecode;
mod projects;
mod lastfm;
mod templates;

#[derive(Debug)]
struct Message {
    id: i32,
    author: String,
    content: String,
    timestamp: DateTime<Utc>
}

struct App {
    pub rate_limiter: RateLimiter,
    pub db_connection: Connection,

    pub projects: Vec<Project>,
    pub weather_cache: Arc<RwLock<String>>,
}

impl App {
    pub fn new() -> Result<Self, ()> {
        let db_connection = Connection::open("guestbook.db")
            .map_err(|e| eprintln!("ERROR: couldn't connect to database: {e}"))?; 

        db_connection.execute(
            "CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                author TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp TEXT NOT NULL
            )",
            []
        ).map_err(|e| eprintln!("ERROR: couldn't execute on database: {e}"))?;

        let projects = load_projects("./static/projects.toml")?;
        let weather_cache = Arc::new(RwLock::new("Loading weather...".to_string()));
        let cache_clone = weather_cache.clone();

        std::thread::spawn(move || {loop {
            match ureq::get("http://wttr.in/Eindhoven?format=2").call() {
                Ok(res) => {
                    let Ok(text) = res.into_body().read_to_string() else { continue; }; 
                    let Ok(mut w) = cache_clone.write() else { continue; };
                    *w = text;
                },
                Err(e) => eprintln!("Failed to update weather: {e}"),
            }

            std::thread::sleep(Duration::from_secs(60*15));
        }});

        Ok(Self {
            rate_limiter: RateLimiter::new(Duration::from_secs(10)),
            db_connection,
            projects,
            weather_cache,
        })
    }
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

fn get_client_ip(request: &Request) -> Option<IpAddr> {
    if let Some(ip) = request.headers().iter()
        .find(|h| h.field.equiv("CF-Connecting-IP"))
        .and_then(|h| h.value.as_str().parse::<IpAddr>().ok()) 
    {
        return Some(ip);
    }
    request.headers().iter()
        .find(|h| h.field.equiv("X-Forwarded-For"))
        .and_then(|h| h.value.as_str().split(',').next())
        .and_then(|ip_str| ip_str.trim().parse::<IpAddr>().ok())
        .or_else(|| request.remote_addr().map(|addr| addr.ip()))
}
    
fn handle_fragment(mut request: Request, app_lock: Arc<Mutex<App>>) -> Result<(), ()> {
    let method = request.method();
    let url = request.url().split('?').next().unwrap_or("");

    let (title, template) = match (method, url) {
        (Method::Get, "/" | "/home") => ("Home", templates::home()),
        (Method::Get, "/guestbook")  => ("Guestbook", templates::guestbook()),
        (Method::Get, "/projects")   => ("Projects", {
            let app = app_lock.lock().unwrap();
            let last_id = request.url().split('?')
                .nth(1)
                .and_then(|q| q.strip_prefix("last_id="))
                .and_then(|v| usize::from_str(v).ok());

            templates::projects(&app.projects, last_id, 10)
        }),
        (Method::Get, "/interests")  => ("Interests", templates::interests()),

        (Method::Get, "/now-playing")  => ("Now Playing",  templates::now_playing()),
        (Method::Get, "/current-time") => ("Current Time", templates::current_time()),
        (Method::Get, "/weather")      => {
            let app = app_lock.lock().unwrap();
            let weather_data = app.weather_cache.read().unwrap();
            ("Weather", templates::weather(&weather_data))
        },
        (Method::Get, "/host-uptime")  => ("Host Uptime",  templates::host_uptime()),

        (Method::Get, "/guestbook/messages") => ("Guestbook Messages", {
            let app = app_lock.lock().unwrap();
            let last_id = request.url().split('?')
                .nth(1)
                .and_then(|q| q.strip_prefix("last_id="))
                .and_then(|v| u32::from_str(v).ok());

            templates::messages(&app.db_connection, last_id, 10)
        }),

        (Method::Post, "/guestbook") => ("Guestbook Messages", {
            let mut app = app_lock.lock().unwrap();
            let ip = get_client_ip(&request);

            let allowed = if let Some(ip) = ip { app.rate_limiter.is_allowed(ip) } else { false };

            if allowed {
                let mut body = String::new();
                request.as_reader().read_to_string(&mut body).unwrap();

                let parsed = urldecode::parse_urlencoded(&body);

                if let (Some(author), Some(content)) = (parsed.get("author"), parsed.get("content")) {
                    let timestamp_str = Local::now().to_rfc3339();
                    let _ = app.db_connection.execute(
                        "INSERT INTO messages (author, content, timestamp) VALUES (?1, ?2, ?3)",
                        &[&author, &content, &timestamp_str]
                    );
                }
                let messages = get_messages(&app.db_connection, None, 1).unwrap_or_default();

                if let Some(msg) = messages.first() {
                    html! {
                        (templates::message(msg))
                        div #form-feedback hx-swap-oob="true" {}
                    }
                } else {
                    html! {}
                }
            } else {
                templates::rate_limit()
            } 

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

fn handle_request(request: Request, app: Arc<Mutex<App>>) -> Result<(), ()> {
    match request.url() {
        url if url.starts_with("/static") => handle_static(request),
        _                                 => handle_fragment(request, app),
    }
}

fn main() -> Result<(), ()> {

    let app = App::new()
        .map_err(|_| eprintln!("ERROR: couldn't create App"))?;
    let app = Arc::new(Mutex::new(app));

    let address = "0.0.0.0:3000";
    let server = Server::http(address)
        .map_err(|e| eprintln!("ERROR: couldn't start server: {e}"))?;

    println!("INFO: started server at {address}");

    let pool = ThreadPool::new(4);
    for request in server.incoming_requests() {
        let app = Arc::clone(&app);
        pool.execute(move || {
            let _ = handle_request(request, app);
        });
    }

    Ok(())
}
