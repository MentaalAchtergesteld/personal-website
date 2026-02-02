use std::{fs, time::Duration};
use maud::{html, Markup};
use chrono::{Utc, DateTime};

use crate::{Message, projects::Project};

fn error(msg: &str, error: impl std::fmt::Display) -> Markup {
    eprintln!("ERROR: {msg}: {error}");
    html! { span { (msg) } }
}

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

pub fn weather(data: Option<&str>) -> Markup {
    lazy_loaded_span(data, "/comp/weather", "load delay:2m")
}

pub fn now_playing(data: Option<&str>) -> Markup {
    lazy_loaded_span(data, "/comp/now-playing", "load delay:30s")
}

pub fn input_form() -> Markup {
    html! {
        form.flex-column
            hx-post="/guestbook"
            hx-target="#message-list"
            hx-swap="afterbegin"
            hx-on::after-request="this.reset()"
        {
            div.flex-row {
                input.border required style="flex-grow: 1;" placeholder="Name" type="text" name="author";
                button type="submit" { "Post!" }
            }
            textarea.border required rows="3" placeholder="Leave a message!" name="content" {}
            div #form-feedback hx-swap-oob="true" {}
        }
    }
}

pub fn form_feedback(title: &str, desc: &str, is_error: bool) -> Markup {
    html! {
        div.border.message.font-small #form-feedback.error[is_error] hx-swap-oob="true" {
            div.title.flex-row.space-between {
                h3 { (title) }
            }
            p { (desc) }
        }
    }
}

pub fn message_item(msg: &Message) -> Markup {
    let is_long = msg.content.chars().count() > 100;

    html! {
        div.border.message.font-small {
            div.title.flex-row.space-between {
                h3 { (msg.author) } 
                span.font-tiny { (smart_time(msg.timestamp)) }
            }
            p.message-content.collapsed[is_long] { (msg.content) }
            
            @if is_long { button.toggle-btn { "Show more" } }
        }
    }
}

pub fn message_list(messages: &[Message], last_id: Option<i32>) -> Markup {
    html! {
        @for msg in messages {
            (message_item(msg))
        }

        @if let Some(last_id) = last_id {
            span #load-more-trigger
                hx-get=(format!("/guestbook/messages?last_id={}", last_id))
                hx-trigger="revealed"
                hx-target="#load-more-trigger"
                hx-swap="outerHTML"
                { "Loading older messages..." }
        }
    }
}

pub fn project_item(project: &Project) -> Markup {
    html! {
        div.border.project.font-small.flex-row {
            div.flex-column.flex-grow style="justify-content: space-between;" {
                div {
                    h3 { (project.title) }
                    p { (project.description) }
                }
                div {
                    a.nav-link href=(project.source_url) target="_blank" rel="noopener noreferrer" { "Source" }
                    @if let Some(deploy_url) = &project.deploy_url {
                        a.nav-link href=(deploy_url) target="_blank" rel="noopener noreferrer" { "Deployed" }
                    }
                }
            }
            @if let Some(image_url) = &project.image_url {
                img style="max-height: 96px;" src=(image_url);
            }
        }
    } 
}

pub fn projects_list(projects: &[Project], last_id: Option<usize>) -> Markup {
    html! {
        @for p in projects {
            (project_item(p))
        }
        @if let Some(last_id) = last_id {
            span #load-more-trigger
                hx-get=(format!("/projects/projects?last_id={}", last_id))
                hx-trigger="revealed"
                hx-target="#load-more-trigger"
                hx-swap="outerHTML"
                { "Loading more projects..." }
        }
    } 
}
