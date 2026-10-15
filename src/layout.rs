use dioxus::prelude::*;

use crate::data::PERSONAL_INFO;
use crate::icons::{DiscordIcon, GithubIcon, MailIcon};
use crate::js;
use crate::route::Route;

#[component]
pub fn Layout() -> Element {
    let current = use_route::<Route>();

    use_effect(|| js::call("smoothScroll"));
    use_effect(use_reactive!(|current| {
        let _ = &current;
        js::call("resetScroll");
    }));

    rsx! {
        header { class: "header",
            h1 { "{PERSONAL_INFO.name}" }
            nav {
                div { class: "links",
                    Link {
                        to: Route::About {},
                        class: if current == (Route::About {}) { "active" } else { "" },
                        "About"
                    }
                    Link {
                        to: Route::Projects {},
                        class: if current == (Route::Projects {}) { "active" } else { "" },
                        "Projects"
                    }
                    Link {
                        to: Route::Blog {},
                        class: if matches!(current, Route::Blog {} | Route::BlogPost { .. }) { "active" } else { "" },
                        "Blog"
                    }
                }
                div { class: "contact",
                    a {
                        href: "{PERSONAL_INFO.github}",
                        aria_label: "GitHub",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        GithubIcon {}
                    }
                    a {
                        href: "{PERSONAL_INFO.discord}",
                        aria_label: "Discord",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        DiscordIcon {}
                    }
                    a {
                        href: "mailto:{PERSONAL_INFO.email}",
                        aria_label: "Email",
                        rel: "external",
                        MailIcon {}
                    }
                }
            }
        }

        main {
            Outlet::<Route> {}
        }
    }
}
