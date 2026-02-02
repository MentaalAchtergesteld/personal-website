use std::{fs, time::Duration};

use chrono::Utc;
use maud::{html, Markup};

pub fn head(title: &str) -> Markup {
    html! {
        title { (title) }
        script src="https://cdn.jsdelivr.net/npm/htmx.org@2.0.6/dist/htmx.min.js" {}
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

pub fn lazy_loaded_span(data: Option<&str>, endpoint: &str, update_interval: &str) -> Markup {
    let (trigger, content) = match data {
        Some(text) => (format!("every {}", update_interval), text),
        None => ("load".to_string(), "loading...".into())
    };
    html! {
        span.center
            hx-get=(endpoint)
            hx-trigger=(trigger)
            hx-swap="outerHTML"
        { (content) }
    }
}

pub fn now_playing(data: Option<&str>) -> Markup {
    lazy_loaded_span(data, "/comp/now-playing", "1m")
}

pub fn server_weather(data: Option<&str>) -> Markup {
    lazy_loaded_span(data, "/comp/server-weather", "5m")
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
