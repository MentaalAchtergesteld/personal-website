use std::{fs::{self}, time::Duration};

use maud::{html, Markup, DOCTYPE};

use crate::App;

fn head(title: &str) -> Markup {
    html! {
        (DOCTYPE)
        meta charset="utf-8";
        title { (title) }
        script src="https://cdn.jsdelivr.net/npm/htmx.org@2.0.6/dist/htmx.min.js" {}
        link rel="stylesheet" href="styles.css";
    }
}

fn footer() -> Markup {
    html! {
        footer.double-border.font-small  {
            p { "Made with " }
            a href="https://www.htmx.org" target="_blank" rel="noopener noreferrer" {
                img src="/img/htmx.svg" alt="HTMX" height="16";
            }
            p { " and "}
            a href="https://htmx.org" target="_blank" rel="noopener noreferrer" {
                img src="/img/rust.svg" alt="Rust" height="16";
            }
        }
    }
}

fn nav_element(name: &str, url: &str) -> Markup {
    html! {
        a.nav-link href=(url)
            hx-get=(url)
            hx-target="#main"
            hx-swap="outerHTML"
            hx-push-url="true"
        { (name) }
    }
}

pub fn navbar() -> Markup {
    html! {
       section.double-border.flex-row.gap8 {
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
                (content)
                (footer())
            }
        }
    }
}

pub fn ascii_banner() -> Result<Markup, ()> {
    let ascii = fs::read_to_string(format!("./static/ascii.txt"))
        .map_err(|e| eprintln!("ERROR: couldn't open file `./static/ascii.txt`: {e}"))?;

    Ok(html! { pre.ascii-banner { (ascii) } })
}

pub fn welcome_message() -> Result<Markup, ()> {
    let message = fs::read_to_string(format!("./static/welcome.txt"))
        .map_err(|e| eprintln!("ERROR: couldn't open file `./static/welcome.txt`: {e}"))?;

    Ok(html! { marquee.welcome-message scrollamount="5" { (message) } })
}

pub fn welcome() -> Result<Markup, ()> {
    Ok(html! {
        section.double-border {
            (ascii_banner()?)
            (welcome_message()?)
        }
    })
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
    html! {
        marquee.bordered 
            scrollamount="2"
            behavior="alternate"
        {
            span 
                hx-get="/api/now-playing"
                hx-trigger="load, every 1s"
                hx-swap="innerHTML"
            {
                "Loading now playing..."
            }
        }
 }
}

pub fn current_time() -> Markup {
    let now = chrono::Local::now();
    let formatted = format!("⏲ {}", now.format("%H:%M:%S"));
    html! {
        span.bordered.center 
            hx-get="/current-time"
            hx-trigger="load delay:1s"
            hx-swap="outerHTML"
        {
            (formatted)
        }
    }
}

pub fn weather() -> Result<Markup, ()> {
    let response = ureq::get("https://wttr.in/Eindhoven?format=2")
        .call()
        .map_err(|e| eprintln!("ERROR: couldn't get weather data: {e}"))?;
    let weather = response.into_body().read_to_string()
        .map_err(|e| eprintln!("ERROR: couldn't read response body into string: {e}"))?;

    Ok(html! {
        span.bordered.center {
            (weather)
        }
    })
}

pub fn host_uptime() -> Result<Markup, ()> {
    let full_uptime = fs::read_to_string("/proc/uptime")
                .map_err(|e| println!("ERROR: couldn't read uptime: {e}"))?;
            let uptime = full_uptime.split_whitespace().next().unwrap_or("0");
            let dur = Duration::from_secs_f64(uptime.parse::<f64>().unwrap_or(0.0));

            let secs = dur.as_secs();
            let days = secs / 86400;
            let hours = (secs % 86400) / 3600;
            let minutes = (secs % 3600) / 60;
            

            let formatted = format!("⏱ Host Uptime: {} days, {} hours, {} minutes", days, hours, minutes);

    Ok(html! {
        span.bordered.center
            hx-get="/host-uptime"
            hx-trigger="load delay:1s"
            hx-swap="outerHTML"
        {
            (formatted)
        }
    })
}

pub fn home() -> Result<Markup, ()> {
    Ok(page("Home", html! {
            (welcome()?)
            img .border src="/img/underconstruction.gif";
            section.double-border.flex-column.gap8 {
                div.flex-row.gap8.align-center.font-small {
                    img src="/img/rattlesnake.gif";
                    (bulletpoint_about())
                }
                div.flex-row.gap4 {
                    (now_playing())
                    (current_time())
                }
                div.flex-row.gap4 {
                    (weather()?)
                    (host_uptime()?)
                }
            }
    }))
}

pub fn guestbook() -> Markup {
    page("Guestbook", html! {
        img .border src="/img/underconstruction.gif";
    })
}

pub fn projects() -> Markup {
    page("Projects", html! {
        img .border src="/img/underconstruction.gif";
    })
}

pub fn interests() -> Markup {
    page("Interests", html! {
        img .border src="/img/underconstruction.gif";
    })
}
