use dioxus::prelude::*;

use crate::data::PERSONAL_INFO;
use crate::js;
use crate::prose::Prose;
use crate::route::Route;

#[component]
pub fn About() -> Element {
    use_effect(|| js::call("typewrite"));

    rsx! {
        document::Title { "{PERSONAL_INFO.name}" }
        article { class: "typing",
            p {
                Prose { text: format!("Hi there! I'm {}, a software developer. I build libraries and tools in Rust, from an ORM and a Discord bot to macros and a docs framework for ", PERSONAL_INFO.name) }
                a { href: "https://dioxuslabs.com", target: "_blank", rel: "noopener noreferrer", "Dioxus" }
                Prose { text: ". This site is built with Dioxus too.", end: true }
            }
            p {
                Prose { text: "I've used TypeScript professionally, and I've tried Zig, Elixir, C++ and Kotlin along the way.", end: true }
            }
            p {
                Prose { text: "What I've built so far is on the " }
                Link { to: Route::Projects {}, "projects" }
                Prose { text: " page, and the rest is on " }
                a { href: "{PERSONAL_INFO.github}", target: "_blank", rel: "noopener noreferrer", "GitHub" }
                Prose { text: "." }
            }
            p {
                Prose { text: "If you want to get in touch, email me at " }
                a { href: "mailto:{PERSONAL_INFO.email}", rel: "external", "{PERSONAL_INFO.email}" }
                Prose { text: " or find me on " }
                a { href: "{PERSONAL_INFO.discord}", target: "_blank", rel: "noopener noreferrer", "Discord" }
                Prose { text: " as meetzli.", end: true }
            }
        }
    }
}
