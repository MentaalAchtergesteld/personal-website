use maud::{html, Markup};

use crate::ui::components;

pub fn home() -> Markup {
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
                    { (components::now_playing(None)) }
                span.center.border { (components::server_clock()) }
            }
            div.flex-row.gap4 {
                span.center.border { (components::server_weather(None)) }
                span.center.border.flex-grow { (components::server_uptime()) }
            }

            (components::socials())
            (components::badge_wall())
        }
        section.border.flex-row.justify-center.gap8 {
            img src="static/img/linuxflipping.gif";
            img src="static/img/gator.gif";
            img src="static/img/eu.gif";
        }
    }
}

pub fn guestbook() -> Markup {
    html! {
        img.border.flex-grow src="static/img/underconstruction.gif";
        section.double-border.flex-column.gap8.justify-center {
            (components::input_form())
            div #message-container
                hx-get="/comp/messages"
                hx-trigger="load"
                hx-swap="innerHTML"
            {
                (components::skeleton_message())
                (components::skeleton_message())
                (components::skeleton_message())
                (components::skeleton_message())
                (components::skeleton_message())
            }
        }
    }
}

pub fn admin(
    admin_path: &str,
    is_authenticated: bool,
    error: Option<&str>,
    csrf_token: Option<&str>,
) -> Markup {
    html! {
        section.double-border.flex-column.gap8 {
            h1 { "Admin" }

            @if is_authenticated {
                form method="post" action=(format!("{admin_path}/logout")) {
                    input type="hidden" name="csrf_token" value=(csrf_token.unwrap_or(""));
                    button type="submit" { "Log out" }
                }
            } @else {
                @if let Some(error) = error {
                    p.error { (error) }
                }

                form.flex-column.gap8 method="post" action=(format!("{admin_path}/login")) {
                    input.border required type="password" name="password"
                        autocomplete="current-password" placeholder="Admin password";
                    button type="submit" { "Log in" }
                }
            }
        }
    }
}

pub fn projects() -> Markup {
    html! {
        img.border.flex-grow src="static/img/underconstruction.gif";
        section.double-border.flex-column.gap8.justify-center {
            div #projects-container
                hx-get="/comp/projects"
                hx-trigger="load"
                hx-swap="innerHTML"
            { "Loading projects..." }
        }
    }
}

pub fn interests() -> Markup {
    html! {
        img.border.flex-grow src="static/img/underconstruction.gif";
        section.double-border.flex-column.gap8.justify-center {
            h1.center { "Jellyfin stats" }
            (components::jellyfin_stats())
        }
    }
}

pub fn not_found() -> Markup {
    html! {
        section.double-border.flex-column.align-center.gap4 {
            h1.center { "Not Found" }
            img src="static/img/dassen.png";
        }
    }
}
