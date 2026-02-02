use maud::{DOCTYPE, Markup, html};

use crate::ui::layout::{head, navbar, footer};

mod layout;
pub mod components;
pub mod pages;

pub fn render(title: &str, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
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
}
