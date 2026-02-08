use std::{collections::HashMap, fs::File, io::Read, path::Path, str::FromStr, sync::{Arc, Mutex}};

use maud::{Markup, html};
use tiny_http::{Header, Method, Request, Response};
use url::form_urlencoded;

use crate::{models::Project, state::App, ui::{self, components, pages::{self, not_found}}, util::{parse_query, rate_limiter::get_client_ip}};

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

fn process_post_message(req: &mut Request, app: Arc<App>) -> Markup {
    let is_allowed = get_client_ip(&req)
        .map_or(false, |ip| app.rate_limiter.lock().unwrap().is_allowed(ip));

    if !is_allowed {
        return components::form_feedback("Rate limited", "You're being too fast! Try again in a few seconds.", true);
    }

    let mut body = String::new();
    if let Err(e) = req.as_reader().read_to_string(&mut body) {
        eprintln!("ERROR: Couldn't read body: {e}");
        return components::form_feedback("Error while posting", "The server could not read the given data.", true)
    }

    let params: HashMap<String, String> = form_urlencoded::parse(body.as_bytes())
        .into_owned()
        .collect();

    let author = params.get("author").map(|s| s.as_str()).unwrap_or("Anonymous");
    let content = params.get("content").map(|s| s.as_str()).unwrap_or("");

    if content.trim().is_empty() {
        return components::form_feedback("Content empty", "No content supplied", false);
    }

    let Ok(msg) = app.message_db.lock().unwrap().create_message(author, content) else {
        return components::form_feedback("Error while creating message", "The server could not create your message", true);
    };

    html! {
        (components::message_item(&msg));
        (components::empty_form_feedback());
    }
}

pub fn handle_comp(mut req: Request, app: Arc<App>) -> Result<(), ()> {
    let method = req.method();
    let url = req.url().split("?").next().unwrap_or("");


    let content = match (method, url) {
        (Method::Get, "/comp/now-playing") => {
            let data = app.lastfm_cache.now_playing.get_or_update(|| Some(app.lastfm.get_now_playing()));
            components::now_playing(data.as_deref())
        },
        (Method::Get, "/comp/top-artists") => {
            let data = app.lastfm_cache.top_artists.get_or_update(|| Some(app.lastfm.get_top_artists(10, "1month")));
            components::top_artists(data.as_deref())
        },
        (Method::Get, "/comp/top-tracks")  => {
            let data = app.lastfm_cache.top_tracks.get_or_update(|| Some(app.lastfm.get_top_tracks(10, "1month")));
            components::top_tracks(data.as_deref())
        }
        (Method::Get, "/comp/top-albums")  => {
            let data = app.lastfm_cache.top_albums.get_or_update(|| Some(app.lastfm.get_top_albums(10, "1month")));
            components::top_albums(data.as_deref())
        }
        (Method::Get, "/comp/user-stats")  => {
            let data = app.lastfm_cache.user_stats.get_or_update(|| app.lastfm.get_user_stats());
            components::lastfm_user_stats(data.as_deref())
        }
        (Method::Get, "/comp/server-weather") => {
            let data = app.wttr_cache.weather.get_or_update(|| Some(app.wttr.get_weather()));
            components::server_weather(data.as_deref())
        }
        (Method::Get, "/comp/projects") => {
            let queries = parse_query(req.url());

            let start_index = queries.get("last_id")
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(0);
            let limit = queries.get("limit")
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(5);

            let (projects, next_index) = if start_index >= app.projects.len() {
                (&[] as &[Project], None)
            } else {
                let end = app.projects.len().min(start_index+limit);
                let slice = &app.projects[start_index..end];
                let next_index = if end < app.projects.len() { Some(end) } else { None };
                (slice, next_index)
            };

            components::projects_list(projects, next_index)
        },
        (Method::Get, "/comp/messages") => {
            let queries = parse_query(req.url());

            let start_index = queries.get("last_id")
                .map(|v| v.parse::<i64>().ok())
                .unwrap_or(None);

            let limit = queries.get("limit")
                .and_then(|v| v.parse::<i64>().ok())
                .unwrap_or(5);

            let messages = app.message_db.lock().unwrap()
                .read_messages(start_index, limit)
                .unwrap_or(vec![]);

            let next_index = if messages.len() as i64 == limit {
                messages.last().map(|msg| msg.id)
            } else { None };

            components::message_list(&messages, next_index)
        },
        (Method::Post, "/comp/messages") => process_post_message(&mut req, app),
        _ => {

            let body = not_found().into_string();
            let response = Response::from_string(body)
                .with_header(Header::from_str("Content-Type: text/html; charset=utf-8").unwrap())
                .with_status_code(404);
            return send_response(req, response)
        }
    };

    let body = content.into_string();
    let response = Response::from_string(body)
        .with_header(Header::from_str("Content-Type: text/html; charset=utf-8").unwrap());

    send_response(req, response)
}

pub fn handle_request(req: Request, app: Arc<App>) -> Result<(), ()> {
    if req.url().starts_with("/static") {return handle_static(req)};
    if req.url().starts_with("/comp")   {return handle_comp(req, app)};

    let method = req.method();
    let url = req.url().split("?").next().unwrap_or("");

    let (title, content, status) = match (method, url) {
        (Method::Get, "/" | "/home") => ("Home",      pages::home(), 200),
        (Method::Get, "/guestbook") =>  ("Guestbook", pages::guestbook(), 200),
        (Method::Get, "/projects") =>   ("Projects",  pages::projects(), 200),
        (Method::Get, "/interests") =>  ("Interests", pages::interests(), 200),
        _ => ("Not Found", pages::not_found(), 404),
    };

    let is_htmx = req.headers().iter().any(|h| h.field.equiv("HX-Request"));

    let body = if is_htmx {
        content.into_string()
    } else {
        ui::render_full(title, content).into_string()
    };

    let response = Response::from_string(body)
        .with_header(Header::from_str("Content-Type: text/html; charset=utf-8").unwrap())
        .with_status_code(status);

    send_response(req, response)
}
