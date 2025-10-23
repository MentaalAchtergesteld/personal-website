use std::{fs::{self}, time::Duration};

use chrono::{DateTime, Datelike, Local, Utc};
use maud::{html, Markup, DOCTYPE};
use rusqlite::{params_from_iter, Connection};

use crate::{lastfm, projects::Project, Message};

// GLOBAL

pub fn not_found() -> Markup {
    html! {
        h1 { "404 Not Found" }
    }
}

fn head(title: &str) -> Markup {
    html! {
        (DOCTYPE)
        meta charset="utf-8";
        title { (title) }
        script src="https://cdn.jsdelivr.net/npm/htmx.org@2.0.6/dist/htmx.min.js" {}
        link rel="stylesheet" href="static/style/styles.css";
    }
}

fn footer() -> Markup {
    html! {
        footer.double-border.font-small  {
            p { "Made with " }
            a href="https://www.htmx.org" target="_blank" rel="noopener noreferrer" {
                img src="static/img/htmx.svg" alt="HTMX" height="16";
            }
            p { " and "}
            a href="https://htmx.org" target="_blank" rel="noopener noreferrer" {
                img src="static/img/rust.svg" alt="Rust" height="16";
            }
        }
    }
}

fn nav_element(name: &str, url: &str) -> Markup {
    html! {
        a.nav-link href=(url) { (name) }
    }
}

pub fn navbar() -> Markup {
    html! {
        section.double-border.flex-row.gap8
            hx-boost="true"
            hx-target="#content"
            hx-push-url="true"
        {
            (nav_element("Home",      "/home"))
            (nav_element("Guestbook", "/guestbook"))
            (nav_element("Projects",  "/projects"))
            (nav_element("Interests", "/interests"))
        } 
    }
}

pub fn page(title: &str, content: Markup) -> Markup {
    html! {
        (head(title))
        body.fg-purple.bg-black {
            section #main {
                (navbar())
                div #content { (content) }
                (footer())
            }
        }
    }
}

// HOME

fn load_text_from_file(path: &str) -> Option<String> {
    fs::read_to_string(path)
        .map_err(|e| eprintln!("ERROR: couldn't open file `{path}`: {e}"))
        .ok()
}

pub fn ascii_banner() -> Markup {
    let ascii = match load_text_from_file("./static/ascii.txt") {
        Some(ascii) => ascii,
        None => "Couldn't load banner".into()
    };
    html! { pre.ascii-banner { (ascii) } }
}

pub fn welcome_message() -> Markup {
    let message = match load_text_from_file("./static/welcome.txt") {
        Some(message) => message,
        None => "Couldn't load message".into()
    };
    html! { marquee.welcome-message scrollamount="5" { (message) } }
}

pub fn welcome() -> Markup  {
    html! {
        section.double-border {
            (ascii_banner())
            (welcome_message())
        }
    }
}

pub fn bulletpoint_about() -> Markup {
    html! {
        div.flex-column {
					span { "~/mentaalachtergesteld"	}
          span { "♫ The Gizzverse Is Reel"	}
          span { "☢ Professional Monster Addict"	}
          span { "λ Schizophrenic Linux User"	}
          span { "☺ Self-Hating Rustacean"	}
          span { "⌨ Scared of anything but a terminal"	}
          span { "✏ Strong minds leave projects unfinished"	}
          span { "★ Sleep Is Fake" }
          }
    }
} 

pub fn now_playing() -> Markup {
    let now_playing = match lastfm::get_now_playing() {
        Ok(np) => np,
        Err(_) => "Couldn't get currently playing song".to_string(),
    };

    html! {
        span
            hx-get="/now-playing"
            hx-trigger="load delay:1s"
            hx-swap="outerHTML"
        {
            (now_playing)
        }
 }
}

pub fn current_time() -> Markup {
    let now = chrono::Local::now();
    let formatted = format!("⏲ {}", now.format("%H:%M:%S"));
    let timestamp_ms = now.timestamp_millis();

    html! {
        span id="server_time" data-server-timestamp=(timestamp_ms)
        {
            (formatted)
        }
        script src="static/script/server_time.js";
    }
}

fn error_span(msg: &str, error: impl std::fmt::Display) -> Markup {
    eprintln!("ERROR: {msg}: {error}");
    html! { span { (msg) } }
}

pub fn weather() -> Markup {
    let response = match ureq::get("https://wttr.in/Eindhoven?format=2").call() {
        Ok(response) => response,
        Err(e) => return error_span("couldn't get weather data", e)
    };

    let weather = match response.into_body().read_to_string() {
        Ok(weather) => weather,
        Err(e) => return error_span("couldn't read weather data", e)
    };

    html! {
        span{
            (weather)
        }
    }
}

pub fn host_uptime() -> Markup {
    let full_uptime = match fs::read_to_string("/proc/uptime") {
        Ok(uptime) => uptime,
        Err(e) => return error_span("couldn't read uptime", e),
    };

    let uptime = full_uptime.split_whitespace().next().unwrap_or("0");
    let dur = Duration::from_secs_f64(uptime.parse::<f64>().unwrap_or(0.0));

    let secs = dur.as_secs();
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;
    

    let formatted = format!("⏱ Host Uptime: {} days, {} hours, {} minutes", days, hours, minutes);

    html! {
        span
            hx-get="/host-uptime"
            hx-trigger="load delay:1s"
            hx-swap="outerHTML"
        {
            (formatted)
        }
    }
}

pub fn home() -> Markup {
    html! {
            (welcome())
            img.border src="static/img/underconstruction.gif";
            section.double-border.flex-column.gap8 {
                div.flex-row.gap8.align-center.font-small {
                    img src="static/img/rattlesnake.gif";
                    (bulletpoint_about())
                }
                div.flex-row.gap4 {
                    marquee.flex-row.center.border.flex-grow
                        scrollamount="2"
                        behavior="alternate"
                        hx-get="/now-playing" hx-trigger="load" hx-swap="innerHTML"  { "loading now playing..." }
                    span.center.border
                        hx-get="/current-time" hx-trigger="load" hx-swap="innerHTML" { "loading current time..." }
                }
                div.flex-row.gap4 {
                    span.center.border
                        hx-get="/weather" hx-trigger="load" hx-swap="innerHTML"      { "loading weather..." } 
                    span.center.border.flex-grow
                        hx-get="/host-uptime" hx-trigger="load" hx-swap="innerHTML"  { "loading uptime..." }
                }
            }
            section.border.flex-row.justify-center {
                img src="static/img/linuxflipping.gif";
                img src="static/img/gator.gif";
                img src="static/img/yugoflag.gif";
            }
    }
}

// GUESTBOOK

fn format_timestamp(timestamp: DateTime<Utc>) -> String {
    let local_ts = timestamp.with_timezone(&Local);
    let now = Local::now();

    let time_str = local_ts.format("%H:%M");

    let ts_date = local_ts.date_naive();

    let today = now.date_naive();

    if ts_date == today {
        format!("Today, {}", time_str)
    } else if Some(ts_date) == today.pred_opt() {
        format!("Yesterday, {}", time_str)
    } else {
        format!("{}-{:02}, {}", local_ts.year(), local_ts.month(), time_str)
    }
}

pub fn get_messages(conn: &Connection, last_id: Option<u32>, limit: u32) -> rusqlite::Result<Vec<Message>> {
    let (sql, params) = match last_id {
        Some(id) => (
            "SELECT id, author, content, timestamp FROM messages WHERE id < ? ORDER BY id DESC LIMIT ?",
            vec![id, limit]
        ),
        None => (
            "SELECT id, author, content, timestamp FROM messages ORDER BY id DESC LIMIT ?",
            vec![limit]
        )
    };

    let mut stmt = conn.prepare(sql)?;
    let iter = stmt.query_map(params_from_iter(params), |row| {
        let timestamp_str: String = row.get(3)?;
        let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or(Utc::now());

        Ok(Message {
            id: row.get(0)?,
            author: row.get(1)?,
            content: row.get(2)?,
            timestamp
        })
    })?;

    iter.collect()
}

pub fn message(message: &Message)-> Markup {
    html! {
        div.border.message.font-small {
            div.title.flex-row.space-between {
                h3 { (message.author) }
                span.font-tiny { (format_timestamp(message.timestamp)) }
            }
            p { (message.content) }
        }
    }
}

pub fn messages(conn: &Connection, last_id: Option<u32>, limit: u32) -> Markup {
    let messages = get_messages(conn, last_id, limit).unwrap_or_default();

    let next_last_id = messages.last().map(|msg| msg.id);

    html! {
            @for msg in &messages {
                (message(msg))
            }

            @if let Some(next_id) = next_last_id {
                span #load-message
                    hx-get=(format!("/guestbook/messages?last_id={}", next_id))
                    hx-trigger="revealed"
                    hx-target="#load-message"
                    hx-swap="outerHTML"
                    {}
            }
    }
}

pub fn message_input() -> Markup {
    html! {
        form.flex-column hx-post="/guestbook" hx-target="#messages" hx-swap="afterbegin" {
            div.flex-row {
                input.border required style="flex-grow: 1;" placeholder="Name" type="text" name="author";
                button type="submit" { "Post!" }
            }
            textarea.border required rows="3" placeholder="Leave a message!" name="content" {}
        }
    }
}

pub fn guestbook() -> Markup {
    html! {
        img .border src="static/img/underconstruction.gif";
        section.double-border.flex-column.gap16 {
            (message_input()) 
            div.flex-column.gap4 #messages 
                hx-get="/guestbook/messages"
                hx-trigger="load"
                hx-swap="innerHTML"
            { span { "Loading messages..." } }
        }
    }
}

// PROJECTS
pub fn project(project: &Project) -> Markup {
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

pub fn projects(projects: &Vec<Project>, last_id: Option<usize>, limit: usize) -> Markup {
    let last_id = last_id.unwrap_or_default();
    let limit = limit.min(projects.len());
    let selected_projects = &projects[last_id..limit];

    let next_id = last_id + limit + 1;
    html! {
        img .border src="static/img/underconstruction.gif";

        @for p in selected_projects {
            (project(p))
        }

        @if projects.len() >= next_id {
            span #load-message
                hx-get=(format!("/guestbook/messages?last_id={}", next_id))
                hx-trigger="revealed"
                hx-target="#load-message"
                hx-swap="outerHTML"
                {}
        }
    }
}

// INTERESTS

pub fn interests() -> Markup {
    html! {
        img .border src="static/img/underconstruction.gif";
    }
}
