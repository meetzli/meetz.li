use yawn_router::{use_location, use_navigate};

component NavLink(href: &'static str, section: &'static str, children: Children) {
    let location = use_location();
    let navigate = use_navigate();
    let active = memo(move || {
        let path = location.get().path;
        let path = path.trim_end_matches('/');
        if section.is_empty() {
            path.is_empty()
        } else {
            path == section || path.starts_with(&format!("{section}/"))
        }
    });

    view {
        <a href={href} class:active={active.get()} on:click={move |e: Event| {
            if e.modified {
                return;
            }
            e.prevent_default();
            navigate.push(href);
        }}>
            <slot />
        </a>
    }
}
