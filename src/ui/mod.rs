use maud::{DOCTYPE, Markup, html};

pub mod components;
pub mod pages;

const NAVBAR_ITEMS: [(&str, &str); 4] = [
    ("/home",      "Home"),
    ("/guestbook", "Guestbook"),
    ("/projects",  "Projects"),
    ("/interests", "Interests"),
];

pub fn render_full(title: &str, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            (components::head(title))
        }
        body {
            section.flex-column #main {
                (components::navbar(&NAVBAR_ITEMS))
                section.flex-column #content { (content) }
                (components::footer())
            }
        }
    }
}
