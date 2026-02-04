use std::{fs, time::Duration};

use chrono::Utc;
use maud::{Markup, PreEscaped, html};

use crate::api::lastfm::{Album, Artist, Track, UserStats};

pub fn head(title: &str) -> Markup {
    html! {
        title { (title) }
        script src="https://cdn.jsdelivr.net/npm/htmx.org@2.0.6/dist/htmx.min.js" {}
        script src="static/script/message.js" {}
        script src="static/script/server_time.js" {}
        link rel="stylesheet" href="static/style/styles.css";
        link rel="icon" type="image/x-icon" href="static/img/favicon.ico";
    }
}

pub fn footer() -> Markup {
    html! {
        footer.double-border.font-small.flex-row  {
            p { "Made with " }
            a href="https://www.htmx.org" target="_blank" rel="noopener noreferrer" {
                img src="static/img/htmx.svg" alt="HTMX" height="16";
            }
            p { " and "}
            a href="https://rust-lang.org/" target="_blank" rel="noopener noreferrer" {
                img src="static/img/rust.svg" alt="Rust" height="16";
            }
        }
    }
}

fn nav_item(endpoint: &str, name: &str) -> Markup {
    html! {
        a .nav-link href=(endpoint) { (name) }
    }
}

pub fn navbar(items: &[(&str, &str)]) -> Markup {
    html! {
        section .double-border.flex-row.gap8
            hx-boost="true"
            hx-target="#content"
            hx-push-url="true"
        { @for item in items { (nav_item(item.0, item.1)) } }
    }
}

fn load_text_from_file(path: &str) -> Option<String> {
    fs::read_to_string(path)
        .map_err(|e| eprintln!("ERROR: couldn't open file `{path}`: {e}"))
        .ok()
}

pub fn ascii_banner() -> Markup {
    let banner = load_text_from_file("./static/ascii.txt").unwrap_or("Couldn't load banner.".into());
    html! { pre.ascii-banner { (banner) } }
}

pub fn welcome_message() -> Markup {
    let message = load_text_from_file("./static/welcome.txt").unwrap_or("Couldn't load welcome message".into());
    html! { marquee.welcome-message scrollamount="5" { (message) } }
}

pub fn bulletpoints() -> Markup {
    let bulletpoints = load_text_from_file("./static/bulletpoints.txt")
        .unwrap_or("☹ Couldn't load bulletpoints".into());

    html! {
        div.flex-column {
            @for point in bulletpoints.lines() {
                span { (point) } 
            }
        }
    }
}

pub fn socials() -> Markup {
    html! {
        div.flex-row.gap4 {
            a.center.border.flex-grow href="https://tidal.com/artist/64262665" target="_blank" rel="noopener noreferrer" {
                img src="static/img/tidal.svg" alt="HTMX" height="24";
            }

            a.center.border.flex-grow href="https://x.com/achtergesteld" target="_blank" rel="noopener noreferrer" {
                img src="static/img/x.svg" alt="HTMX" height="24";
            }

            a.center.border.flex-grow href="https://twitch.tv/mentaalachtergesteld" target="_blank" rel="noopener noreferrer" {
                img src="static/img/twitch.svg" alt="HTMX" height="24";
            }

            a.center.border.flex-grow href="https://github.com/mentaalachtergesteld" target="_blank" rel="noopener noreferrer" {
                img src="static/img/github.svg" alt="HTMX" height="24";
            }
        }
    }
}

// DATA COMPONENTS

pub fn lazy_component(
    content: Option<Markup>,
    endpoint: &str,
    interval: &str,
    is_inline: bool
) -> Markup {
    let (trigger, inner_html) = match content {
        Some(html) => (format!("every {interval}"), html),
        None => ("load".to_string(), html! { "loading..." })
    };

    let tag = if is_inline { "span" } else { "div" };

    html! {
        (PreEscaped(format!("<{} class='center' hx-get='{}' hx-trigger='{}' hx-swap='outerHTML'>", tag, endpoint, trigger)))
            (inner_html)
        (PreEscaped(format!("</{}>", tag)))
    }
}

pub fn now_playing(data: Option<&Option<Track>>) -> Markup {
    let content = data.map(|track_opt| match track_opt {
        Some(track) => html! { "♫ Now Playing: " (track.name) " by " (track.artist) " ♫" },
        None => html! { "☹ Nothing playing right now ☹" }
    });

    lazy_component(content, "/comp/now-playing", "1m", true)
}

pub fn top_artists(data: Option<&Vec<Artist>>) -> Markup {
    let content = data.map(|ta| html! {
        div.flex-column.gap4 { @for (i, artist) in ta.iter().enumerate() {
            div.list-row {
                span.rank-col { (i+1) "."}
                span.truncate title=(artist.name) { (artist.name) }
            }
        }} 
    });

    lazy_component(content, "/comp/top-artists", "", false)
}

pub fn top_tracks(data: Option<&Vec<Track>>) -> Markup {
    let content = data.map(|tt| html! {
        div.flex-column.gap4 { @for (i, track) in tt.iter().enumerate() {
            div.list-row {
                span.rank-col { (i+1) "." }
                span.truncate title=(track.name) { (track.name) }
                span { "-" }
                span.truncate title=(track.artist) { (track.artist) }
            }
        }}
    });

    lazy_component(content, "/comp/top-tracks", "", false)
}

pub fn top_albums(data: Option<&Vec<Album>>) -> Markup {
    let content = data.map(|ta| html! {
        div.flex-column.gap4 { @for (i, album) in ta.iter().enumerate() {
            div.list-row {
                span.rank-col { (i+1) "." } 
                span.truncate title=(album.name) { (album.name) } 
                span { "-" }
                span.truncate title=(album.artist) { (album.artist) } 
            }
        }}
    });

    lazy_component(content, "/comp/top-albums", "", false)
}

pub fn lastfm_user_stats(data: Option<&UserStats>) -> Markup {
    let content = data.map(|us| html! {
        div.flex-column.gap4 {
            span { "Total scrobbles: " (us.total_scrobbles) }
            span { "Total tracks: " (us.total_tracks) }
            span { "Total artists: " (us.total_artists) }
            span { "Total albums: " (us.total_albums) }
        }
    });

    lazy_component(content, "/comp/user-stats", "", false)
}

pub fn lastfm_stats() -> Markup {
    html! {
        div.flex-row.gap4.justify-center {
            div.flex-column.gap4.w50 {
                div.align-center.border {
                    div.flex-row.align-center.justify-center.gap4 {
                        h1 { "Top Artists " }
                        span.font-tiny { "(1 month)" }
                    }
                    (top_artists(None))
                }
                div.align-center.border {
                    div.flex-row.align-center.justify-center.gap4 {
                        h1 { "Top Tracks" }
                        span.font-tiny { "(1 month)" }
                    }
                    (top_tracks(None))
                }
            }
            div.flex-column.gap4.w50 {
                div.align-center.border {
                    div.flex-row.align-center.justify-center.gap4 {
                        h1 { "User Stats" }
                    }
                    (lastfm_user_stats(None))
                }
                div.align-center.border {
                    div.flex-row.align-center.justify-center.gap4 {
                        h1 { "Top Albums" }
                        span.font-tiny { "(1 month)" }
                    }
                    (top_albums(None))
                }
            }
        }
    }

}

pub fn server_weather(data: Option<&String>) -> Markup {
    let content = data.map(|t| html! { (t) });
    lazy_component(content, "/comp/server-weather", "5m", true)
}

pub fn server_clock() -> Markup {
    let ts_millis = Utc::now().timestamp_millis();
    html! { span.live-time data-ts=(ts_millis) data-type=("clock") { "⏲ loading..." } }
}

pub fn server_uptime() -> Markup {
    let full_uptime = match fs::read_to_string("/proc/uptime") {
        Ok(uptime) => uptime,
        Err(_) => return html! { span {"Couldn't read uptime" } },
    };

    let seconds_str = full_uptime.split_whitespace().next().unwrap_or("0");
    let seconds = seconds_str.parse().unwrap_or(0.0);
    let duration = Duration::from_secs(seconds as u64);
    let boot_time = Utc::now() - duration;
    let ts_millis = boot_time.timestamp_millis();

    html! { span.live-time data-ts=(ts_millis) data-type="uptime" { "⏱ loading..." } }
}
