use dioxus::prelude::*;

use crate::layout::Layout;
use crate::pages::{About, Blog, BlogPost, NotFound, Projects};

#[derive(Routable, Clone, PartialEq, Debug)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Layout)]
    #[route("/")]
    About {},
    #[route("/projects")]
    Projects {},
    #[route("/blog")]
    Blog {},
    #[route("/blog/:slug")]
    BlogPost { slug: String },
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
}
