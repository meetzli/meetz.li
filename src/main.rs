mod data;
mod highlight;
mod icons;
mod js;
mod layout;
mod pages;
mod prose;
mod route;
mod typography;

use dioxus::prelude::*;
use route::Route;

fn main() {
    launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Meta { name: "theme-color", content: "#1d2021" }

        Router::<Route> {}
    }
}
