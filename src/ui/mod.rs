use maud::{DOCTYPE, Markup, html};

use crate::ui::layout::{head, navbar, footer};

mod layout;
pub mod components;
pub mod pages;

pub enum RenderType {
    Partial,
    Full
}

pub fn render(title: &str, content: Markup, render_type: RenderType) -> String {
    match render_type {
        RenderType::Partial => content.into_string(),
        RenderType::Full => {
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
            }.into_string()
        }
    }
}
