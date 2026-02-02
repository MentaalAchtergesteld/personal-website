use std::{fs::File, io::Read, path::Path, str::FromStr, sync::{Arc, Mutex}};

use tiny_http::{Header, Method, Request, Response};

use crate::{state::App, ui::{self, pages}};

fn send_response<R: Read>(req: Request, res: Response<R>) -> Result<(), ()> {
    req.respond(res)
        .map_err(|e| eprintln!("ERROR: Couldn't respond: {e}"))
}

fn handle_static(req: Request) -> Result<(), ()> {
    let rel_path = req.url().trim_start_matches("/");
    if rel_path.contains("..") {
        return send_response(req, Response::empty(404))
    }

    let path = Path::new("./").join(rel_path);

    if !path.exists() || !path.is_file() {
        return send_response(req, Response::empty(404)) 
    }

    let file = File::open(&path).map_err(|e| eprintln!("ERROR: Couldn't open file: {e}"))?;

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

    send_response(req, response)
}

pub fn handle_request(req: Request, app: Arc<Mutex<App>>) -> Result<(), ()> {
    if req.url().starts_with("/static") {return handle_static(req)};

    let method = req.method();
    let url = req.url().split("?").next().unwrap_or("");

    let (title, content) = match (method, url) {
        (Method::Get, "/" | "/home") => ("Home",      pages::home()),
        (Method::Get, "/guestbook") =>  ("Guestbook", pages::guestbook()),
        (Method::Get, "/projects") =>   ("Projects",  pages::projects()),
        (Method::Get, "/interests") =>  ("Interests", pages::interests()),
        _ => ("404", pages::not_found())
    };

    let is_htmx = req.headers().iter().any(|h| h.field.equiv("HX-Request"));

    let body = if is_htmx {
        content.into_string()
    } else {
        ui::render_full(title, content).into_string()
    };

    let response = Response::from_string(body)
        .with_header(Header::from_str("Content-Type: text/html; charset=utf-8").unwrap());

    send_response(req, response)
}
