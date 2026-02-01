use std::{fs, time::Duration};
use maud::{html, Markup};
use chrono::{Utc, DateTime};

fn error(msg: &str, error: impl std::fmt::Display) -> Markup {
    eprintln!("ERROR: {msg}: {error}");
    html! { span { (msg) } }
}

// TIME

pub fn clock() -> Markup {
    let ts_millis = Utc::now().timestamp_millis();
    html! { span.live-time data-ts=(ts_millis) data-type=("clock") { "⏲ loading..." } }
}

pub fn uptime() -> Markup {
    let full_uptime = match fs::read_to_string("/proc/uptime") {
        Ok(uptime) => uptime,
        Err(e) => return error("couldn't read uptime", e),
    };

    let seconds_str = full_uptime.split_whitespace().next().unwrap_or("0");
    let seconds = seconds_str.parse().unwrap_or(0.0);
    let duration = Duration::from_secs(seconds as u64);
    let boot_time = Utc::now() - duration;
    let ts_millis = boot_time.timestamp_millis();

    html! { span.live-time data-ts=(ts_millis) data-type="uptime" { "⏲ loading..." } }
}

pub fn smart_time(timestamp: DateTime<Utc>) -> Markup {
    let ts_millis = timestamp.timestamp_millis();
    html! { span.live-time data-ts=(ts_millis) data-type="smart" {} }
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

pub fn lazy_loaded_span(data: Option<&str>, link: &str, trigger: &str) -> Markup {
    let text = data.unwrap_or("loading...");
    html! {
        span
            hx-get=(link)
            hx-trigger=(trigger)
            hx-swap="outerHTML"
        { (text) }
    }
}

pub fn weather(data: Option<&str>) -> Markup {
    lazy_loaded_span(data, "/comp/weather", "load delay:2m")
}

pub fn now_playing(data: Option<&str>) -> Markup {
    lazy_loaded_span(data, "/comp/now-playing", "load delay:30s")
}
