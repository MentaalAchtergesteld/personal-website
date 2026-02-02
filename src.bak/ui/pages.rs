use maud::{html, Markup};

use crate::{App, lastfm, ui::components::{self}};

pub fn home(app: &App) -> Markup {
    let now_playing_data = lastfm::get_now_playing().unwrap_or("Couldn't load current song.".into());
    let weather_guard = app.weather_cache.read();
    let weather_data = match &weather_guard {
        Ok(g) => g.as_str(),
        Err(_) => "Error loading weather."
    };

    html! {
        section.double-border.flex-column.gap8 {
            (components::ascii_banner())
            (components::welcome_message())
        }
        img.border.flex-grow src="static/img/underconstruction.gif";
        section.double-border.flex-column.gap8 {
            div.flex-row.gap8.align-center.font-small {
                img src="static/img/rattlesnake.gif";
                (components::bulletpoints())
            }
            div.flex-row.gap4 {
                marquee.flex-row.center.border.flex-grow
                    scrollamount="2" behavior="alternate"
                    { (components::now_playing(Some(&now_playing_data))) }
                span.center.border { (components::clock()) }
            }
            div.flex-row.gap4 {
                span.center.border { (components::weather(Some(weather_data))) } 
                span.center.border.flex-grow { (components::uptime()) }
            }
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
        section.border.flex-row.justify-center {
            img src="static/img/linuxflipping.gif";
            img src="static/img/gator.gif";
            img src="static/img/yugoflag.gif";
        }
    }
}

pub fn guestbook() -> Markup {
    html! {
        img .border src="static/img/underconstruction.gif";
        section.double-border.flex-column.gap16 {
            (components::input_form()) 
            div.flex-column.gap4 #message-list
                hx-get="/guestbook/messages"
                hx-trigger="load"
                hx-swap="innerHTML"
            { span { "Loading messages..." } }
        }
    }
}

pub fn projects() -> Markup {
    html! {
        img.border src="static/img/underconstruction.gif";
        section.double-border.flex-column.gap16 {
            div.flex-column.gap4 #project-list
                hx-get="/projects/projects"
                hx-trigger="load"
                hx-swap="innerHTML"
            { span { "Loading projects..." } }
        }
    }
}

pub fn interests() -> Markup {
    html! {
    img.border src="static/img/underconstruction.gif";
        section.double-border.flex-column.gap16 {
            h1 { "Interests" }
        }
    }
}
