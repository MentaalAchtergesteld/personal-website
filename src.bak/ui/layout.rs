use maud::{html, Markup};

pub fn head(title: &str) -> Markup {
    html! {
        title { (title) }
        script src="https://cdn.jsdelivr.net/npm/htmx.org@2.0.6/dist/htmx.min.js" {}
        script src="static/script/server_time.js" {}
        script src="static/script/message.js" {}
        link rel="stylesheet" href="static/style/styles.css";
        link rel="icon" type="image/x-icon" href="static/img/favicon.ico";
    }
}

fn nav_item(name: &str, url: &str) -> Markup {
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
            (nav_item("Home",      "/home"))
            (nav_item("Guestbook", "/guestbook"))
            (nav_item("Projects",  "/projects"))
            (nav_item("Interests", "/interests"))
        } 
    }
}
pub fn footer() -> Markup {
    html! {
        footer.double-border.font-small  {
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
