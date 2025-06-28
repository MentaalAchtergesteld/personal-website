use std::{collections::HashMap, fs::File, str::FromStr};
use chrono::{DateTime, Local, Utc};
use maud::Markup;
use spotify::SpotifyAPI;
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};

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

fn serve_file(request: Request, path: &str) -> Result<(), ()> {
    let page = File::open(format!("./static/{path}"))
        .map_err(|e| eprintln!("ERROR: couldn't open file `{path}`: {e}"))?;
    let response = Response::from_file(page);

    let _ = request.respond(response);
    Ok(())
}

fn serve_template(request: Request, template: Markup) -> Result<(), ()> {
    let response = Response::from_string(template)
        .with_header(Header::from_str("Content-Type: text/html").unwrap());

    request.respond(response)
        .map_err(|e| eprintln!("ERROR: couldn't serve template: {e}"))
}

fn parse_form(data: String) -> HashMap<String, String> {
    HashMap::new()
}

fn handle_api(mut request: Request, app: &mut App) -> Result<(), ()> {
    let method = request.method();
    let url = request.url().strip_prefix("/api").unwrap_or("");

    match (method, url) {
        (Method::Get, "/now-playing") => {
            let now_playing = app.spotify_api.get_now_playing().unwrap_or("Error fetching now playing".to_string());
            let response = Response::from_string(now_playing);

            let _ = request.respond(response);
            Ok(())
        },
        (Method::Post, "/guestbook") => {
            let mut body = String::new();
            request.as_reader().read_to_string(&mut body).unwrap();

            let parsed = url::form_urlencoded::parse(body.as_bytes())
                .into_owned()
                .collect::<HashMap<String, String>>();

            if let (Some(author), Some(content)) = (parsed.get("author"), parsed.get("content")) {
                app.messages.push(Message {
                    author: author.to_string(),
                    content: content.to_string(),
                    timestamp: Local::now().into() 
                }); 
                serve_template(request, templates::messages(&app.messages))
            } else {
                let _ = request.respond(Response::empty(StatusCode(400)));
                Ok(())
            }
        } 
        _ => {
            let response = Response::from_string("Not Found").with_status_code(StatusCode(404));

            let _ = request.respond(response);
            Ok(())
        },
    }
}

fn handle_request(request: Request, app: &mut App) -> Result<(), ()> {
    let method = request.method();
    let url = request.url().to_string();
    match (method, url.as_str()) {
        (_, url) if url.starts_with("/api") => handle_api(request, app),
        (Method::Get, "/" | "/home")   => serve_template(request, templates::home()?),
        (Method::Get, "/guestbook")    => serve_template(request, templates::guestbook(&app)),
        (Method::Get, "/interests")    => serve_template(request, templates::interests()),
        (Method::Get, "/projects")     => serve_template(request, templates::projects()),

        (Method::Get, "/now-playing")  => serve_template(request, templates::now_playing()),
        (Method::Get, "/current-time") => serve_template(request, templates::current_time()),
        (Method::Get, "/host-uptime")  => serve_template(request, templates::host_uptime()?),
        (Method::Get, url) => {
            let path = url.strip_prefix("/").unwrap_or("");

            if path.contains("..") {
                let response = Response::from_string("Forbidden").with_status_code(403);
                let _ = request.respond(response);
                Err(())
            } else {
                serve_file(request, path)
            }
        },
        _ => Ok(())
    }
}

fn main() -> Result<(), ()> {
    let mut app = App {
        spotify_api: SpotifyAPI::new(),
        messages: Vec::new(),
    };

    let address = "127.0.0.1:3000";
    let server = Server::http(address)
        .map_err(|e| eprintln!("ERROR: couldn't start server: {e}"))?;

    println!("INFO: started server at {address}");

    for request in server.incoming_requests() {
        let _ = handle_request(request, &mut app);
    }

    Ok(())
}
