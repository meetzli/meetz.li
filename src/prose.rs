use dioxus::prelude::*;

use crate::typography::{bind_end, tie};

#[component]
pub fn Prose(text: String, #[props(default = false)] end: bool) -> Element {
    let text = tie(&text);
    let text = if end { bind_end(&text) } else { text };

    rsx! { {text} }
}
