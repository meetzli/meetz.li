use yawn_router::{use_location, Link};

component NotFound() {
    yawn::ui::set_title("Not found | meetzli");
    let location = use_location();
    let path = memo(move || location.get().path.trim_matches('/').to_string());

    view {
        <article>
            <p>
                "There is nothing at /{path}. Try the "
                <Link href="/">"about page"</Link>
                " or the "
                <Link href="/projects">"projects"</Link>
                "."
            </p>
        </article>
    }
}
