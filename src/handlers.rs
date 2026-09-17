use std::{collections::HashMap, fs::File, io::Read, path::Path, str::FromStr, sync::Arc};

use maud::{html, Markup};
use tiny_http::{Header, Method, Request, Response};
use url::form_urlencoded;

use crate::{
    auth::AdminAuth,
    models::Project,
    state::App,
    ui::{
        self, components,
        pages::{self, not_found},
    },
    util::{parse_query, rate_limiter::get_client_ip},
};

const ADMIN_COOKIE: &str = "admin_session";
const MAX_ADMIN_FORM_BYTES: u64 = 4 * 1024;

fn send_response<R: Read>(req: Request, res: Response<R>) -> Result<(), ()> {
    req.respond(res)
        .map_err(|e| eprintln!("ERROR: Couldn't respond: {e}"))
}

fn get_cookie(req: &Request, name: &str) -> Option<String> {
    req.headers()
        .iter()
        .find(|header| header.field.equiv("Cookie"))?
        .value
        .as_str()
        .split(';')
        .filter_map(|cookie| cookie.trim().split_once('='))
        .find_map(|(cookie_name, value)| (cookie_name == name).then(|| value.to_string()))
}

fn admin_session(req: &Request, app: &App) -> Option<(String, String)> {
    let auth = app.admin_auth.as_ref()?;
    let token = get_cookie(req, ADMIN_COOKIE)?;
    let csrf_token = auth.session_csrf_token(&token)?;
    Some((token, csrf_token))
}

fn session_cookie(auth: &AdminAuth, token: &str) -> String {
    let secure = if auth.uses_secure_cookie() {
        "; Secure"
    } else {
        ""
    };
    format!(
        "{ADMIN_COOKIE}={token}; Path=/; HttpOnly; SameSite=Strict; Max-Age={}{}",
        auth.session_ttl().as_secs(),
        secure
    )
}

fn expired_session_cookie(auth: &AdminAuth) -> String {
    let secure = if auth.uses_secure_cookie() {
        "; Secure"
    } else {
        ""
    };
    format!("{ADMIN_COOKIE}=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0{secure}")
}

fn read_form(req: &mut Request, max_bytes: u64) -> Result<HashMap<String, String>, ()> {
    let mut body = String::new();
    req.as_reader()
        .take(max_bytes + 1)
        .read_to_string(&mut body)
        .map_err(|e| eprintln!("ERROR: Couldn't read form body: {e}"))?;

    if body.len() as u64 > max_bytes {
        return Err(());
    }

    Ok(form_urlencoded::parse(body.as_bytes())
        .into_owned()
        .collect())
}

fn send_admin_page(
    req: Request,
    admin_path: &str,
    is_authenticated: bool,
    error: Option<&str>,
    csrf_token: Option<&str>,
    status: u16,
) -> Result<(), ()> {
    let body = ui::render_full(
        "Admin",
        pages::admin(admin_path, is_authenticated, error, csrf_token),
    )
    .into_string();

    let response = Response::from_string(body)
        .with_header(Header::from_str("Content-Type: text/html; charset=utf-8").unwrap())
        .with_header(Header::from_str("Cache-Control: no-store").unwrap())
        .with_header(Header::from_str("X-Robots-Tag: noindex, nofollow, noarchive").unwrap())
        .with_status_code(status);

    send_response(req, response)
}

fn handle_admin_login(mut req: Request, app: Arc<App>) -> Result<(), ()> {
    let Some(auth) = app.admin_auth.as_ref() else {
        return send_response(req, Response::empty(404));
    };

    let is_allowed = req.remote_addr().is_some_and(|address| {
        app.admin_login_rate_limiter
            .lock()
            .unwrap()
            .is_allowed(address.ip())
    });

    if !is_allowed {
        return send_admin_page(
            req,
            auth.path(),
            false,
            Some("Too many login attempts. Try again shortly."),
            None,
            429,
        );
    }

    let Ok(params) = read_form(&mut req, MAX_ADMIN_FORM_BYTES) else {
        return send_admin_page(
            req,
            auth.path(),
            false,
            Some("Invalid login request."),
            None,
            400,
        );
    };

    let password = params.get("password").map(String::as_str).unwrap_or("");
    if !auth.verify_password(password) {
        return send_admin_page(
            req,
            auth.path(),
            false,
            Some("Invalid password."),
            None,
            401,
        );
    }

    let token = auth.create_session().map_err(|e| {
        eprintln!("ERROR: Couldn't create admin session: {e}");
    })?;

    let response = Response::empty(303)
        .with_header(Header::from_str(&format!("Location: {}", auth.path())).unwrap())
        .with_header(
            Header::from_str(&format!("Set-Cookie: {}", session_cookie(auth, &token))).unwrap(),
        )
        .with_header(Header::from_str("Cache-Control: no-store").unwrap());

    send_response(req, response)
}

fn handle_admin_logout(mut req: Request, app: Arc<App>) -> Result<(), ()> {
    let Some(auth) = app.admin_auth.as_ref() else {
        return send_response(req, Response::empty(404));
    };
    let Some((session_token, expected_csrf)) = admin_session(&req, &app) else {
        return send_response(req, Response::empty(401));
    };
    let Ok(params) = read_form(&mut req, MAX_ADMIN_FORM_BYTES) else {
        return send_response(req, Response::empty(400));
    };

    if params.get("csrf_token").map(String::as_str) != Some(expected_csrf.as_str()) {
        return send_response(req, Response::empty(403));
    }

    auth.revoke_session(&session_token);

    let response = Response::empty(303)
        .with_header(Header::from_str(&format!("Location: {}", auth.path())).unwrap())
        .with_header(
            Header::from_str(&format!("Set-Cookie: {}", expired_session_cookie(auth))).unwrap(),
        )
        .with_header(Header::from_str("Cache-Control: no-store").unwrap());

    send_response(req, response)
}

fn process_delete_message(req: Request, app: Arc<App>, id: i64) -> Result<(), ()> {
    let Some((_, expected_csrf)) = admin_session(&req, &app) else {
        return send_response(req, Response::empty(401));
    };

    let supplied_csrf = req
        .headers()
        .iter()
        .find(|header| header.field.equiv("X-CSRF-Token"))
        .map(|header| header.value.as_str());

    if supplied_csrf != Some(expected_csrf.as_str()) {
        return send_response(req, Response::empty(403));
    }

    let result = app
        .message_db
        .lock()
        .map_err(|e| eprintln!("ERROR: Message database lock poisoned: {e}"))?
        .delete_message(id);

    match result {
        Ok(true) => send_response(req, Response::from_string("")),
        Ok(false) => send_response(req, Response::empty(404)),
        Err(e) => {
            eprintln!("ERROR: Couldn't delete message {id}: {e}");
            send_response(req, Response::empty(500))
        }
    }
}

fn handle_static(req: Request) -> Result<(), ()> {
    let rel_path = req.url().trim_start_matches("/");
    if rel_path.contains("..") {
        return send_response(req, Response::empty(404));
    }

    let path = Path::new("./").join(rel_path);

    if !path.exists() || !path.is_file() {
        return send_response(req, Response::empty(404));
    }

    let file = File::open(&path).map_err(|e| eprintln!("ERROR: Couldn't open file: {e}"))?;

    let content_type = match path.extension().and_then(|ext| ext.to_str()) {
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        _ => "application/octet-stream",
    };

    let response = Response::from_file(file)
        .with_header(Header::from_str(&format!("Content-Type: {content_type}")).unwrap())
        .with_header(Header::from_str("Cache-Control: public, max-age=86400").unwrap());

    send_response(req, response)
}

fn process_post_message(req: &mut Request, app: Arc<App>) -> Markup {
    let admin_csrf_token = admin_session(req, &app).map(|(_, csrf_token)| csrf_token);
    let is_allowed =
        get_client_ip(&req).map_or(false, |ip| app.rate_limiter.lock().unwrap().is_allowed(ip));

    if !is_allowed {
        return components::form_feedback(
            "Rate limited",
            "You're being too fast! Try again in a few seconds.",
            true,
        );
    }

    let mut body = String::new();
    if let Err(e) = req.as_reader().read_to_string(&mut body) {
        eprintln!("ERROR: Couldn't read body: {e}");
        return components::form_feedback(
            "Error while posting",
            "The server could not read the given data.",
            true,
        );
    }

    let params: HashMap<String, String> = form_urlencoded::parse(body.as_bytes())
        .into_owned()
        .collect();

    let author = params
        .get("author")
        .map(|s| s.as_str())
        .unwrap_or("Anonymous");
    let content = params.get("content").map(|s| s.as_str()).unwrap_or("");

    if content.trim().is_empty() {
        return components::form_feedback("Content empty", "No content supplied", false);
    }

    let Ok(msg) = app
        .message_db
        .lock()
        .unwrap()
        .create_message(author, content)
    else {
        return components::form_feedback(
            "Error while creating message",
            "The server could not create your message",
            true,
        );
    };

    html! {
        (components::message_item(&msg, admin_csrf_token.as_deref()));
        (components::empty_form_feedback());
    }
}

fn get_cached_music_stats(app: &App) -> Option<Arc<crate::api::jellyfin::MusicStats>> {
    app.jellyfin_cache.music_stats.get_or_update(|| {
        app.jellyfin
            .get_music_stats()
            .map_err(|error| eprintln!("ERROR: Couldn't get Jellyfin music stats: {error}"))
            .ok()
    })
}

pub fn handle_comp(mut req: Request, app: Arc<App>) -> Result<(), ()> {
    let path = req.url().split("?").next().unwrap_or("").to_string();

    if req.method() == &Method::Delete {
        let Some(id) = path
            .strip_prefix("/comp/messages/")
            .filter(|value| !value.contains('/'))
            .and_then(|value| value.parse::<i64>().ok())
        else {
            return send_response(req, Response::empty(404));
        };

        return process_delete_message(req, app, id);
    }

    let method = req.method();
    let url = path.as_str();

    let content = match (method, url) {
        (Method::Get, "/comp/now-playing") => {
            let data = app.jellyfin_cache.now_playing.get_or_update(|| {
                match app.jellyfin.get_now_playing() {
                    Ok(track) => Some(track),
                    Err(error) => {
                        eprintln!("ERROR: Couldn't get Jellyfin sessions: {error}");
                        None
                    }
                }
            });
            match data {
                Some(data) => components::now_playing(Some(&data)),
                None => components::api_error(
                    "Jellyfin unavailable; retrying...",
                    "/comp/now-playing",
                    true,
                ),
            }
        }
        (Method::Get, "/comp/top-artists") => {
            let data = get_cached_music_stats(&app);
            match data {
                Some(data) => components::top_artists(Some(&data.top_artists)),
                None => components::api_error(
                    "Jellyfin unavailable; retrying...",
                    "/comp/top-artists",
                    false,
                ),
            }
        }
        (Method::Get, "/comp/top-tracks") => {
            let data = get_cached_music_stats(&app);
            match data {
                Some(data) => components::top_tracks(Some(&data.top_tracks)),
                None => components::api_error(
                    "Jellyfin unavailable; retrying...",
                    "/comp/top-tracks",
                    false,
                ),
            }
        }
        (Method::Get, "/comp/top-albums") => {
            let data = get_cached_music_stats(&app);
            match data {
                Some(data) => components::top_albums(Some(&data.top_albums)),
                None => components::api_error(
                    "Jellyfin unavailable; retrying...",
                    "/comp/top-albums",
                    false,
                ),
            }
        }
        (Method::Get, "/comp/user-stats") => {
            let data = get_cached_music_stats(&app);
            match data {
                Some(data) => components::jellyfin_user_stats(Some(&data.user_stats)),
                None => components::api_error(
                    "Jellyfin unavailable; retrying...",
                    "/comp/user-stats",
                    false,
                ),
            }
        }
        (Method::Get, "/comp/server-weather") => {
            let data = app
                .wttr_cache
                .weather
                .get_or_update(|| Some(app.wttr.get_weather()));
            components::server_weather(data.as_deref())
        }
        (Method::Get, "/comp/projects") => {
            let queries = parse_query(req.url());

            let start_index = queries
                .get("last_id")
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(0);
            let limit = queries
                .get("limit")
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(5);

            let (projects, next_index) = if start_index >= app.projects.len() {
                (&[] as &[Project], None)
            } else {
                let end = app.projects.len().min(start_index + limit);
                let slice = &app.projects[start_index..end];
                let next_index = if end < app.projects.len() {
                    Some(end)
                } else {
                    None
                };
                (slice, next_index)
            };

            components::projects_list(projects, next_index)
        }
        (Method::Get, "/comp/messages") => {
            let queries = parse_query(req.url());

            let start_index = queries
                .get("last_id")
                .map(|v| v.parse::<i64>().ok())
                .unwrap_or(None);

            let limit = queries
                .get("limit")
                .and_then(|v| v.parse::<i64>().ok())
                .unwrap_or(5);

            let messages = app
                .message_db
                .lock()
                .unwrap()
                .read_messages(start_index, limit)
                .unwrap_or(vec![]);

            let next_index = if messages.len() as i64 == limit {
                messages.last().map(|msg| msg.id)
            } else {
                None
            };

            let admin_csrf_token = admin_session(&req, &app).map(|(_, csrf_token)| csrf_token);
            components::message_list(&messages, next_index, admin_csrf_token.as_deref())
        }
        (Method::Post, "/comp/messages") => process_post_message(&mut req, app),
        _ => {
            let body = not_found().into_string();
            let response = Response::from_string(body)
                .with_header(Header::from_str("Content-Type: text/html; charset=utf-8").unwrap())
                .with_status_code(404);
            return send_response(req, response);
        }
    };

    let body = content.into_string();
    let response = Response::from_string(body)
        .with_header(Header::from_str("Content-Type: text/html; charset=utf-8").unwrap())
        .with_header(Header::from_str("Cache-Control: no-store").unwrap());

    send_response(req, response)
}

pub fn handle_request(req: Request, app: Arc<App>) -> Result<(), ()> {
    if req.url().starts_with("/static") {
        return handle_static(req);
    };
    if req.url().starts_with("/comp") {
        return handle_comp(req, app);
    };

    let path = req.url().split("?").next().unwrap_or("");
    if req.method() == &Method::Post
        && app
            .admin_auth
            .as_ref()
            .is_some_and(|auth| auth.is_login_path(path))
    {
        return handle_admin_login(req, app);
    }
    if req.method() == &Method::Post
        && app
            .admin_auth
            .as_ref()
            .is_some_and(|auth| auth.is_logout_path(path))
    {
        return handle_admin_logout(req, app);
    }

    let method = req.method();
    let url = req.url().split("?").next().unwrap_or("");

    let admin_session = admin_session(&req, &app);

    let (title, content, status) = match (method, url) {
        (Method::Get, "/" | "/home") => ("Home", pages::home(), 200),
        (Method::Get, "/guestbook") => ("Guestbook", pages::guestbook(), 200),
        (Method::Get, "/projects") => ("Projects", pages::projects(), 200),
        (Method::Get, "/interests") => ("Interests", pages::interests(), 200),
        (Method::Get, path)
            if app
                .admin_auth
                .as_ref()
                .is_some_and(|auth| auth.path() == path) =>
        {
            (
                "Admin",
                pages::admin(
                    app.admin_auth.as_ref().unwrap().path(),
                    admin_session.is_some(),
                    None,
                    admin_session
                        .as_ref()
                        .map(|(_, csrf_token)| csrf_token.as_str()),
                ),
                200,
            )
        }
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
    let response = if app
        .admin_auth
        .as_ref()
        .is_some_and(|auth| auth.path() == url)
    {
        response
            .with_header(Header::from_str("Cache-Control: no-store").unwrap())
            .with_header(Header::from_str("X-Robots-Tag: noindex, nofollow, noarchive").unwrap())
    } else {
        response
    };

    send_response(req, response)
}
