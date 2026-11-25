use crate::data::PERSONAL_INFO;
use crate::typography::{bind_end, tie};
use yawn_router::Link;

component Index() {
    yawn::ui::set_title(PERSONAL_INFO.name);
    let intro = tie(&format!(
        "Hi there! I'm {}, a software developer. I build libraries and tools in Rust, from an ORM and a Discord bot to macros and a docs framework for ",
        PERSONAL_INFO.name
    ));

    view {
        <article class="typing">
            <p>
                {intro.clone()}
                <a href="https://dioxuslabs.com" target="_blank" rel="noopener noreferrer">"Dioxus"</a>
                {tie(". This site is built with ")}
                <a href="https://github.com/yawn-rs/yawn" target="_blank" rel="noopener noreferrer">"Yawn"</a>
                {bind_end(" too.")}
            </p>
            <p>{bind_end(&tie("I've used TypeScript professionally, and I've tried Zig, Elixir, C++ and Kotlin along the way."))}</p>
            <p>
                {tie("What I've built so far is on the ")}
                <Link href="/projects">"projects"</Link>
                {tie(" page, and the rest is on ")}
                <a href={PERSONAL_INFO.github} target="_blank" rel="noopener noreferrer">"GitHub"</a>
                "."
            </p>
            <p>
                {tie("If you want to get in touch, email me at ")}
                <a href="mailto:{PERSONAL_INFO.email}" rel="external">{PERSONAL_INFO.email}</a>
                {tie(" or find me on ")}
                <a href={PERSONAL_INFO.discord} target="_blank" rel="noopener noreferrer">"Discord"</a>
                {bind_end(&tie(" as meetzli."))}
            </p>
        </article>
    }
}
