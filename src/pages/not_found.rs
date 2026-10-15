use dioxus::prelude::*;

use crate::route::Route;

#[component]
pub fn NotFound(segments: Vec<String>) -> Element {
    let path = segments.join("/");

    rsx! {
        document::Title { "Not found | meetzli" }
        article {
            p {
                "There is nothing at /{path}. Try the "
                Link { to: Route::About {}, "about page" }
                " or the "
                Link { to: Route::Projects {}, "projects" }
                "."
            }
        }
    }
}
