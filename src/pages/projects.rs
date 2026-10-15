use dioxus::prelude::*;

use crate::data::PROJECTS;
use crate::js;
use crate::prose::Prose;

#[component]
pub fn Projects() -> Element {
    use_effect(|| js::reveal(".project"));

    rsx! {
        document::Title { "Projects | meetzli" }
        article {
            for project in PROJECTS.iter().rev() {
                section { class: "project",
                    span { class: "when", "{project.date}" }
                    h2 {
                        a {
                            href: "{project.link}",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            "{project.name}"
                        }
                    }
                    p { class: "meta", "{project.tech}" }
                    p { Prose { text: project.description.to_string(), end: true } }
                    a {
                        class: "project-preview",
                        href: "{project.link}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        img {
                            src: "{project.image}",
                            alt: "Preview of {project.name}",
                            loading: "lazy",
                            width: "1200",
                            height: "600",
                        }
                    }
                }
            }
        }
    }
}
