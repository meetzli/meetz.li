mod data;
mod highlight;
mod markdown;
mod typography;

use yawn_router::Router;

component App() {
    yawn::ui::head::meta("theme-color", "#1d2021");
    yawn::ui::head::link("icon", "/assets/favicon.ico");

    view {
        <Router routes={routes::table()} />
    }
}
